//! Terminal UI helpers for update/install animations.
//!
//! Provides spinners, progress bars, and styled output for the update flow.
//! These run before the TUI starts, so they use stdout directly.

use std::time::Duration;

use console::{style, Style};
use indicatif::{ProgressBar, ProgressStyle};

/// Spinner frames using braille dots for smooth animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Frame interval for spinner animation (80ms per frame)
const SPINNER_INTERVAL: u64 = 80;

/// Delay between phases (500ms)
const PHASE_DELAY: Duration = Duration::from_millis(500);

/// Create a spinner with the given message.
///
/// Returns a `ProgressBar` configured as a spinner with braille dots.
/// Call `.finish_with_message()` when done.
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(SPINNER_FRAMES)
            .template("  {spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(SPINNER_INTERVAL));
    pb
}

/// Create a progress bar for downloads.
///
/// Shows percentage, bytes transferred, speed, and ETA.
pub fn progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "  [{bar:26.green/dim}] {percent:>3}%\n   {bytes}/{total_bytes}   {wide_msg:.dim}",
            )
            .expect("valid template")
            .progress_chars("━━░"),
    );
    pb
}

/// Print a bordered message box.
///
/// Each line is printed inside a box using Unicode box-drawing characters.
pub fn print_box(lines: &[&str]) {
    // Find the maximum line width
    let max_width = lines
        .iter()
        .map(|l| console::measure_text_width(l))
        .max()
        .unwrap_or(0);
    let box_width = max_width + 4; // 2 chars padding on each side

    let horizontal = "─".repeat(box_width);

    println!();
    println!("  ┌{}┐", horizontal);
    println!("  │{}│", " ".repeat(box_width));

    for line in lines {
        let padding = box_width - console::measure_text_width(line) - 2;
        let left_pad = 1;
        let right_pad = padding - left_pad;
        println!(
            "  │{}{}{}│",
            " ".repeat(left_pad),
            line,
            " ".repeat(right_pad.max(0))
        );
    }

    println!("  │{}│", " ".repeat(box_width));
    println!("  └{}┘", horizontal);
    println!();
}

/// Print a success message with green checkmark.
pub fn print_success(message: &str) {
    println!("  {} {}", style("✓").green(), style(message).green());
}

/// Print a success message with sparkle emoji (for "update found").
pub fn print_sparkle(message: &str) {
    println!(
        "  {} {}",
        style("✨").green(),
        style(message).green().bold()
    );
}

/// Print a success message with checkmark emoji.
pub fn print_check(message: &str) {
    println!("  {} {}", style("✅").green(), style(message).green());
}

/// Print a warning message with warning emoji.
pub fn print_warning(message: &str) {
    let dim_yellow = Style::new().yellow().dim();
    println!("  ⚠️  {}", dim_yellow.apply_to(message));
}

/// Print an error message with optional details.
pub fn print_error(message: &str, details: Option<&str>) {
    println!();
    println!("  {} {}", style("❌").red(), style(message).red().bold());

    if let Some(details) = details {
        println!();
        println!("     {}", details);
    }
}

/// Print an error with a suggestion.
pub fn print_error_with_suggestion(message: &str, error_detail: &str, suggestion: &str) {
    println!();
    println!("  {} {}", style("❌").red(), style(message).red().bold());
    println!();
    println!("     {}", error_detail);
    println!();
    println!("     {}", style(suggestion).dim());
}

/// Sleep for the standard phase delay (500ms).
pub fn delay() {
    std::thread::sleep(PHASE_DELAY);
}

/// Finish a spinner with a success checkmark.
pub fn spinner_success(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {msg}")
            .expect("valid template"),
    );
    pb.finish_with_message(format!("{} {}", style("✓").green(), style(message).green()));
}

/// Finish a spinner with an error.
pub fn spinner_error(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {msg}")
            .expect("valid template"),
    );
    pb.finish_with_message(format!("{} {}", style("✗").red(), style(message).red()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames_count() {
        // Braille spinner should have 10 frames for smooth animation
        assert_eq!(SPINNER_FRAMES.len(), 10);
    }

    #[test]
    fn test_phase_delay() {
        // Phase delay should be 500ms
        assert_eq!(PHASE_DELAY, Duration::from_millis(500));
    }
}
