use arcane_core::db::category::list_categories;
use arcane_core::db::schedule::{list_schedule_overrides, list_schedule_slots, log_session};
use arcane_core::scheduler::resolve_timeline;
use chrono::{Duration, Local, NaiveTime};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use notify_rust::Notification;
use sqlx::SqlitePool;
use std::io::{self, Write};
use std::time::Duration as StdDuration;

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Self {
        let _ = terminal::enable_raw_mode();
        RawModeGuard
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn print_progress(category_name: &str, elapsed: i64, total: i64, paused: bool) {
    let percent = if total > 0 {
        (elapsed * 100) / total
    } else {
        100
    };
    let percent = percent.clamp(0, 100);

    let filled_width = (percent * 20) / 100;
    let mut bar = String::new();
    for i in 0..20 {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push('░');
        }
    }

    let remaining_seconds = (total - elapsed).max(0);
    let rem_minutes = remaining_seconds / 60;
    let rem_secs = remaining_seconds % 60;

    let status = if paused { "PAUSED" } else { "ACTIVE" };

    print!(
        "\r[{}] {}: [{}] {:02}% - {:02}m {:02}s remaining  (Space: Pause, S: Skip, Q: Quit)",
        status, category_name, bar, percent, rem_minutes, rem_secs
    );
    let _ = io::stdout().flush();
}

async fn wait_until_start(category_name: &str, start_time: NaiveTime) -> Result<(), ()> {
    let guard = RawModeGuard::new();

    loop {
        let now = Local::now().time();
        if now >= start_time {
            break;
        }

        let diff = start_time - now;
        let diff_secs = diff.num_seconds();
        let min = diff_secs / 60;
        let sec = diff_secs % 60;

        print!(
            "\rWaiting for next block '{}' starting at {} (in {:02}m {:02}s)... [S] Start now, [Q] Quit",
            category_name,
            start_time.format("%H:%M"),
            min,
            sec
        );
        let _ = io::stdout().flush();

        if event::poll(StdDuration::from_millis(200)).unwrap_or(false)
            && let Ok(Event::Key(key_event)) = event::read()
            && key_event.kind == KeyEventKind::Press
        {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    println!("\nSkipping wait, starting immediately.");
                    break;
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    println!("\nAborted.");
                    drop(guard);
                    return Err(());
                }
                _ => {}
            }
        }
    }

    drop(guard);
    Ok(())
}

async fn run_timer(
    category_name: &str,
    total_seconds: i64,
    notifications_enabled: bool,
) -> Result<Option<i64>, ()> {
    if notifications_enabled {
        let _ = Notification::new()
            .summary("Starting session")
            .body(&format!("Starting session: {}", category_name))
            .show();
    }
    println!("\nStarting session: {}", category_name);

    let guard = RawModeGuard::new();
    let mut elapsed_ms = 0;
    let mut paused = false;

    print_progress(category_name, 0, total_seconds, paused);

    while elapsed_ms < total_seconds * 1000 {
        if event::poll(StdDuration::from_millis(100)).unwrap_or(false)
            && let Ok(Event::Key(key_event)) = event::read()
            && key_event.kind == KeyEventKind::Press
        {
            match key_event.code {
                KeyCode::Char(' ') => {
                    paused = !paused;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    println!("\nSkipped.");
                    drop(guard);
                    return Ok(Some(elapsed_ms / 1000));
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    println!("\nAborted.");
                    drop(guard);
                    return Err(());
                }
                _ => {}
            }
        }

        if !paused {
            elapsed_ms += 100;
        }

        if elapsed_ms % 1000 == 0 || paused {
            print_progress(category_name, elapsed_ms / 1000, total_seconds, paused);
        }
    }

    println!("\nSession complete: {}", category_name);
    if notifications_enabled {
        let _ = Notification::new()
            .summary("Session complete")
            .body(&format!("Session complete: {}", category_name))
            .show();
    }

    drop(guard);
    Ok(Some(total_seconds))
}

pub async fn start_cmd(pool: &SqlitePool, notifications_enabled: bool) {
    let today = Local::now().date_naive();

    let slots = match list_schedule_slots(pool).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error fetching schedule slots: {}", e);
            return;
        }
    };

    let overrides = match list_schedule_overrides(today, pool).await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error fetching schedule overrides: {}", e);
            return;
        }
    };

    let categories = match list_categories(pool).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error fetching categories: {}", e);
            return;
        }
    };

    let timeline = resolve_timeline(today, &slots, &overrides);
    if timeline.is_empty() {
        println!("No tasks scheduled for today.");
        return;
    }

    println!("Today's schedule timeline (resolved):");
    for block in &timeline {
        if let Some(cat) = categories.iter().find(|c| c.id == block.category_id) {
            println!("  - {} | {}", block.time_of_day, cat.name);
        }
    }
    println!();

    for block in timeline {
        let start_time = match NaiveTime::parse_from_str(&block.time_of_day, "%H:%M") {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Invalid time format in block: {}", block.time_of_day);
                continue;
            }
        };

        let cat = match categories.iter().find(|c| c.id == block.category_id) {
            Some(c) => c,
            None => continue,
        };

        let duration = Duration::minutes(cat.default_minutes as i64);
        let end_time = start_time + duration;

        let now = Local::now().time();

        if now >= end_time {
            println!(
                "Skipping completed block '{}' ({} - {})",
                cat.name,
                start_time.format("%H:%M"),
                end_time.format("%H:%M")
            );
            continue;
        }

        let total_seconds;
        if now >= start_time && now < end_time {
            let remaining = end_time - now;
            total_seconds = remaining.num_seconds();
            println!(
                "Catching up with active block '{}' (remaining: {}m {}s)",
                cat.name,
                total_seconds / 60,
                total_seconds % 60
            );
        } else {
            // now < start_time
            if wait_until_start(&cat.name, start_time).await.is_err() {
                return;
            }
            total_seconds = duration.num_seconds();
        }

        let elapsed_secs = match run_timer(&cat.name, total_seconds, notifications_enabled).await {
            Ok(Some(secs)) => secs,
            Ok(None) => continue,
            Err(_) => return,
        };

        if elapsed_secs == 0 {
            continue;
        }

        let start_time_dt = Local::now().naive_local() - Duration::seconds(elapsed_secs);

        print!("Enter notes for this session (optional): ");
        let _ = io::stdout().flush();
        let mut notes_input = String::new();
        let _ = io::stdin().read_line(&mut notes_input);
        let notes_trimmed = notes_input.trim();
        let notes = if notes_trimmed.is_empty() {
            None
        } else {
            Some(notes_trimmed.to_string())
        };

        let mut rating = None;
        loop {
            print!("Score your recall rating (0-5, optional, press Enter to skip): ");
            let _ = io::stdout().flush();
            let mut rating_input = String::new();
            if io::stdin().read_line(&mut rating_input).is_err() {
                break;
            }
            let rating_trimmed = rating_input.trim();
            if rating_trimmed.is_empty() {
                break;
            }
            if let Ok(val) = rating_trimmed.parse::<u32>()
                && val <= 5
            {
                rating = Some(val);
                break;
            }
            println!("Invalid rating. Please enter an integer between 0 and 5.");
        }

        match log_session(
            cat.id,
            start_time_dt,
            (elapsed_secs / 60) as u32,
            notes.as_deref(),
            rating,
            pool,
        )
        .await
        {
            Ok(_) => println!("Session logged successfully."),
            Err(e) => eprintln!("Failed to log session: {}", e),
        }
    }
}
