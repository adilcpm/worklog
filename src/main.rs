use chrono::{Datelike, Duration, TimeZone, Utc};
use clap::{Parser, Subcommand};
use comfy_table::{Cell, Table};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::{fs, path::PathBuf};
use std::{thread, time::Duration as StdDuration};

/// Location helper – ~/.local/share/worklog/log.json (Linux) or OS‑equivalent
fn log_file() -> PathBuf {
    let proj = ProjectDirs::from("com", "example", "worklog").expect("Could not find user dirs");
    let dir = proj.data_local_dir();
    fs::create_dir_all(dir).expect("cannot create data dir");
    dir.join("log.json")
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    tag: String,
    start: i64, // UNIX timestamp (UTC seconds)
    end: Option<i64>,
}

impl Session {
    fn duration(&self) -> Option<Duration> {
        self.end.map(|e| Duration::seconds(e - self.start))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Simple Work‑Hours Logger", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start logging a new activity tagged <TAG>
    Start { tag: String },
    /// Stop the currently running activity
    Stop,
    /// Show current activity status
    Status,
    /// Reset (discard) the current activity without logging it
    Reset,
    /// Show the location of the log file
    Path,
    /// Log custom hours for a task (e.g., "worklog log mytask 2.5")
    Log { tag: String, hours: f64 },
    /// Show a report – default: daily
    Report {
        #[arg(value_parser = ["daily", "weekly", "monthly"], default_value = "daily")]
        period: String,
    },
}

fn load_log() -> Vec<Session> {
    let path = log_file();
    if !path.exists() {
        return Vec::new();
    }
    let data = fs::read_to_string(path).expect("cannot read log");
    serde_json::from_str(&data).unwrap_or_default()
}

fn save_log(log: &[Session]) {
    let data = serde_json::to_string_pretty(log).expect("serialize");
    fs::write(log_file(), data).expect("write log");
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

fn cmd_start(tag: String) {
    let mut log = load_log();
    if log.iter().any(|s| s.end.is_none()) {
        eprintln!("Existing session still running. Stop it first.");
        return;
    }

    // Create and save the session immediately
    let start_time = Utc::now().timestamp();
    log.push(Session {
        tag: tag.clone(),
        start: start_time,
        end: None,
    });
    save_log(&log);

    // Clear screen and show initial message
    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
    println!("🚀 Started: {}", tag);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n Press Ctrl+C to exit display (activity will continue running)\n");

    // Display running timer until interrupted
    loop {
        let duration = Utc::now().timestamp() - start_time;
        print!("\r\x1B[K"); // Clear line
        println!(
            "\r⏱  Time elapsed: \x1B[1;32m{}\x1B[0m",
            format_duration(duration)
        );
        print!("\x1B[1A"); // Move cursor up one line
        io::stdout().flush().unwrap();

        thread::sleep(StdDuration::from_secs(1));
    }
}

fn cmd_stop() {
    let mut log = load_log();
    match log.iter_mut().find(|s| s.end.is_none()) {
        Some(s) => {
            let tag = s.tag.clone();
            s.end = Some(Utc::now().timestamp());
            save_log(&log);
            println!("Stopped {}.", tag);
        }
        None => eprintln!("No running session."),
    }
}

fn cmd_status() {
    let log = load_log();
    match log.iter().find(|s| s.end.is_none()) {
        Some(s) => {
            let duration = Utc::now().timestamp() - s.start;
            let hours = duration as f64 / 3600.0;
            println!("Currently working on: {} ({:.2}h)", s.tag, hours);
        }
        None => println!("No active session."),
    }
}

fn cmd_reset() {
    let mut log = load_log();
    match log.iter().position(|s| s.end.is_none()) {
        Some(pos) => {
            let session = log.remove(pos);
            save_log(&log);
            println!("Reset session: {}", session.tag);
        }
        None => println!("No active session to reset."),
    }
}

fn cmd_path() {
    println!("{}", log_file().display());
}

fn cmd_log(tag: String, hours: f64) {
    if hours <= 0.0 {
        eprintln!("Hours must be positive.");
        return;
    }

    let mut log = load_log();
    let now = Utc::now().timestamp();
    let duration_seconds = (hours * 3600.0) as i64;
    let start_time = now - duration_seconds;

    log.push(Session {
        tag: tag.clone(),
        start: start_time,
        end: Some(now),
    });

    save_log(&log);
    println!("Logged {:.2} hours for '{}'.", hours, tag);
}

fn within_period(ts: i64, period: &str) -> bool {
    let dt = Utc.timestamp_opt(ts, 0).single().unwrap();
    let now = Utc::now();
    match period {
        "daily" => dt.date_naive() == now.date_naive(),
        "weekly" => {
            let w1 = dt.iso_week();
            let w2 = now.iso_week();
            w1.year() == w2.year() && w1.week() == w2.week()
        }
        "monthly" => dt.year() == now.year() && dt.month() == now.month(),
        _ => false,
    }
}

fn cmd_report(period: String) {
    let mut table = Table::new();
    table.set_header(vec!["Tag", "Total (h)"]);

    let log = load_log();

    // Aggregate seconds per tag
    let mut agg: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for s in log.iter().filter(|s| s.end.is_some()) {
        if within_period(s.start, &period) || within_period(s.end.unwrap(), &period) {
            let dur = s.duration().unwrap().num_seconds();
            *agg.entry(s.tag.clone()).or_insert(0) += dur;
        }
    }

    if agg.is_empty() {
        println!("No completed sessions for {} period.", period);
        return;
    }

    let mut pairs: Vec<(String, i64)> = agg.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    for (tag, secs) in pairs {
        let hrs = secs as f64 / 3600.0;
        table.add_row(vec![Cell::new(tag), Cell::new(format!("{:.2}", hrs))]);
    }

    println!("{} report\n{}", period.to_uppercase(), table);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start { tag } => cmd_start(tag),
        Commands::Stop => cmd_stop(),
        Commands::Status => cmd_status(),
        Commands::Reset => cmd_reset(),
        Commands::Path => cmd_path(),
        Commands::Log { tag, hours } => cmd_log(tag, hours),
        Commands::Report { period } => cmd_report(period),
    }
}
