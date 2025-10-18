use mouse_position::mouse_position::Mouse;
use smol::Timer;
use std::time::Duration;
use xcap::Monitor;
use xcap::image::RgbaImage;

const LIMIT: usize = 2;

// Async function to process screenshots list
async fn process_screenshots(screenshots: Vec<Vec<u8>>) {
    println!("Processing {} screenshots", screenshots.len());
    for (i, img) in screenshots.iter().enumerate() {
        println!("Screenshot {} size: {} bytes", i, img.len());
    }
}

fn capture_screenshot() -> Option<RgbaImage> {
    // Capture the first available display's screenshot
    if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
        let monitor = Monitor::from_point(x, y).ok()?;
        return monitor.capture_image().ok();
    }
    None
}

fn main() {
    smol::block_on(async {
        let mut screenshots = Vec::new();

        loop {
            if let Some(img) = capture_screenshot() {
                screenshots.push(img.to_vec());
            } else {
                eprintln!("Failed to capture screenshot");
            }

            if screenshots.len() >= LIMIT {
                process_screenshots(screenshots).await;
                screenshots = Vec::new();
            }

            Timer::after(Duration::from_secs(1)).await;
        }
    });
}
