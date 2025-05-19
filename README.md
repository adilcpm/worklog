# Worklog

A simple command-line tool for tracking work hours and activities. Worklog helps you keep track of time spent on different tasks by logging start and stop times, and generating reports.

## Features

- Start and stop time tracking for activities
- Tag-based activity tracking
- Generate reports for different time periods:
  - Daily
  - Weekly
  - Monthly
- Automatic local data storage
- Clean command-line interface
- Check current activity status

## Installation

Make sure you have Rust and Cargo installed on your system. Then:

```bash
cargo install --path .
```

## Usage

### Start tracking an activity

```bash
worklog start "project-x"
```

This will start tracking the activity and display a running timer. You can press Ctrl+C to exit the display - the activity will continue running in the background. Use `worklog status` to check on it later, or `worklog stop` to end it.

### Stop the current activity

```bash
worklog stop
```

### Check current activity status

```bash
worklog status
```

### Reset (discard) the current activity

```bash
worklog reset
```

### Generate reports

```bash
# Daily report (default)
worklog report

# Weekly report
worklog report weekly

# Monthly report
worklog report monthly
```

### Show log file location

```bash
worklog path
```

## Data Storage

Worklog stores its data in your system's local data directory:

- Linux: `~/.local/share/worklog/log.json`
- macOS: `~/Library/Application Support/worklog/log.json`
- Windows: `%LOCALAPPDATA%\worklog\log.json`

## License

MIT
