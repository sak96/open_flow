use crate::Config;
use crate::action::{ActionEvent, ActionType};
use log::{error, info};
use smol::{Timer, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use xcap::Monitor;

pub async fn screenshot_loop(
    receiver: channel::Receiver<ActionEvent>,
    running: Arc<AtomicBool>,
    config: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut text_buffer = String::new();
    let mut last_screenshot = Instant::now();

    // Get primary monitor
    let monitor = Monitor::all()?
        .into_iter()
        .find(|m| m.is_primary())
        .ok_or("No primary monitor found")?;

    info!(
        "Screenshot loop started. Monitor: {}x{}",
        monitor.width(),
        monitor.height()
    );

    while running.load(Ordering::Relaxed) {
        // Throttle screenshot rate
        let elapsed = last_screenshot.elapsed();
        if elapsed < Duration::from_millis(config.screenshot_throttle_ms) {
            Timer::after(Duration::from_millis(10)).await;
            continue;
        }

        // Capture screenshot
        match monitor.capture_image() {
            Ok(_image) => {
                last_screenshot = Instant::now();
                if config.debug_mode {
                    info!("Screenshot captured");
                }
                // Process or save image here
            }
            Err(e) => {
                error!("Failed to capture: {}", e);
                Timer::after(Duration::from_millis(100)).await;
                continue;
            }
        }

        // Wait for and process actions
        loop {
            match receiver.try_recv() {
                Ok(action) => {
                    if action.is_printable_key() {
                        // Accumulate printable characters
                        if let ActionType::Key { value } = &action.event_type {
                            text_buffer.push_str(value);
                        }
                    } else {
                        // Non-printable event: print buffer and take screenshot
                        if !text_buffer.is_empty() {
                            info!("Text: \"{}\"", text_buffer);
                            text_buffer.clear();
                        }

                        match &action.event_type {
                            ActionType::Key { value } => {
                                info!("Special key: {}", value);
                            }
                            ActionType::Mouse { value, position } => {
                                info!("Mouse {:?} at {:?}", value, position);
                            }
                        }

                        break; // Loop back to take new screenshot
                    }
                }
                Err(channel::TryRecvError::Empty) => {
                    break; // No more events
                }
                Err(channel::TryRecvError::Closed) => {
                    return Ok(());
                }
            }
        }

        Timer::after(Duration::from_millis(10)).await;
    }

    Ok(())
}
