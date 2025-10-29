use crate::action::{ActionEvent, action_loop};
use crate::config::Config;
use crate::screenshot::screenshot_loop;
use log::info;
use smol::{Executor, channel};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

mod action;
mod config;
mod screenshot;

// ==================== MAIN ====================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::default());
    let running = Arc::new(AtomicBool::new(true));

    simple_logger::SimpleLogger::new().env().init().unwrap();
    // Bounded channel with backpressure (capacity: 1000 events)
    let (sender, receiver) = channel::bounded::<ActionEvent>(1000);

    let ex = Executor::new();

    info!("Starting Input Capture System");
    info!("Press Ctrl+C to exit");

    smol::block_on(ex.run(async {
        // Start action loop
        action_loop(sender, running.clone(), config.clone()).await?;

        // Start screenshot loop
        screenshot_loop(receiver, running, config).await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    }))
}
