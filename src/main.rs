use crate::action::{ActionEvent, action_loop};
use crate::config::Config;
use crate::screenshot::screenshot_loop;
use log::info;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;

mod action;
mod config;
mod screenshot;

// ==================== MAIN ====================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::default());
    let running = Arc::new(AtomicBool::new(true));

    simple_logger::SimpleLogger::new().env().init().unwrap();
    // Bounded channel with backpressure (capacity: 1000 events)
    let (sender, receiver) = mpsc::channel::<ActionEvent>(1000);

    info!("Starting Input Capture System");
    info!("Press Ctrl+C to exit");
    tokio::try_join! {
        action_loop(sender, running.clone(), config.clone()),
        screenshot_loop(receiver, running, config),
    }?;
    Ok(())
}
