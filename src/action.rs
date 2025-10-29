use crate::config::Config;
use smol::{Timer, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::thread;

#[cfg(target_os = "linux")]
use evdev::{Device, EventType, InputEvent};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use rdev::{Button, Event, EventType as RdevEventType, listen};

// Unified event structure with timestamp
#[derive(Debug, Clone)]
pub struct ActionEvent {
    pub timestamp: Instant,
    pub event_type: ActionType,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Key {
        value: String,
    },
    Mouse {
        value: MouseButton,
        position: (i32, i32),
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Move,
    Other(u8),
}

impl ActionEvent {
    pub fn new_key(value: String) -> Self {
        Self {
            timestamp: Instant::now(),
            event_type: ActionType::Key { value },
        }
    }

    pub fn new_mouse(button: MouseButton, position: (i32, i32)) -> Self {
        Self {
            timestamp: Instant::now(),
            event_type: ActionType::Mouse {
                value: button,
                position,
            },
        }
    }

    // Check if character is printable (a-z, A-Z, 0-9, punctuation, space)
    pub fn is_printable_key(&self) -> bool {
        match &self.event_type {
            ActionType::Key { value } => {
                value.len() == 1
                    && value
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_graphic() || c == ' ')
                        .unwrap_or(false)
            }
            _ => false,
        }
    }
}

#[cfg(target_os = "linux")]
pub async fn action_loop(
    sender: channel::Sender<ActionEvent>,
    running: Arc<AtomicBool>,
    config: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Enumerate input devices from /dev/input
    let devices: Vec<Device> = evdev::enumerate()
        .filter_map(|(path, dev)| {
            let is_keyboard = dev.supported_keys().is_some();
            let is_mouse = dev.supported_relative_axes().is_some();

            if is_keyboard || is_mouse {
                if config.debug_mode {
                    println!(
                        "Found device: {} - {:?}",
                        dev.name().unwrap_or("unknown"),
                        path
                    );
                }
                Some(dev)
            } else {
                None
            }
        })
        .collect();

    if devices.is_empty() {
        return Err("No input devices found. Add user to 'input' group.".into());
    }

    println!("Monitoring {} input devices", devices.len());

    // Spawn async task for each device
    for mut device in devices {
        let sender_clone = sender.clone();
        let running_clone = running.clone();
        let config_clone = config.clone();

        smol::spawn(async move {
            while running_clone.load(Ordering::Relaxed) {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let Some(action) = linux_event_to_action(&event, &config_clone) {
                                let _ = sender_clone.send(action).await;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading events: {}", e);
                        Timer::after(Duration::from_millis(100)).await;
                    }
                }
            }
        })
        .detach();
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_event_to_action(event: &InputEvent, config: &Config) -> Option<ActionEvent> {
    match event.event_type() {
        EventType::KEY if event.code() >= 272 && event.code() <= 275 => {
            // Mouse buttons: BTN_LEFT=272, BTN_RIGHT=273, BTN_MIDDLE=274
            if event.value() == 0 {
                return None; // Ignore release
            }

            let button = match event.code() {
                272 => MouseButton::Left,
                273 => MouseButton::Right,
                274 => MouseButton::Middle,
                n => MouseButton::Other((n - 272) as u8),
            };

            Some(ActionEvent::new_mouse(button, (0, 0)))
        }
        EventType::KEY => {
            if event.value() == 0 {
                return None; // Ignore key release
            }

            let key_code = evdev::Key::new(event.code());
            let key_name = format!("{:?}", key_code);

            // Strip "KEY_" prefix for cleaner output
            let value = if key_name.starts_with("KEY_") {
                key_name
                    .strip_prefix("KEY_")
                    .unwrap_or(&key_name)
                    .to_string()
            } else {
                key_name
            };

            Some(ActionEvent::new_key(value))
        }
        EventType::RELATIVE if config.enable_mouse_move => {
            // Relative mouse movement
            Some(ActionEvent::new_mouse(MouseButton::Move, (0, 0)))
        }
        _ => None,
    }
}

// ==================== MACOS/WINDOWS ACTION LOOP ====================

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub async fn action_loop(
    sender: channel::Sender<ActionEvent>,
    running: Arc<AtomicBool>,
    config: Arc<Config>,
) {
    // rdev's listen() is blocking, so run in separate thread
    thread::spawn(move || {
        if config.debug_mode {
            println!("Starting rdev listener...");
        }

        let _ = listen(move |event: Event| {
            if !running.load(Ordering::Relaxed) {
                return;
            }

            if let Some(action) = rdev_event_to_action(&event, &config) {
                let _ = sender.try_send(action);
            }
        });
    });
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn rdev_event_to_action(event: &Event, config: &Config) -> Option<ActionEvent> {
    match event.event_type {
        RdevEventType::KeyPress(key) => {
            let key_str = format!("{:?}", key);
            Some(ActionEvent::new_key(key_str))
        }

        RdevEventType::ButtonPress(button) => {
            let mouse_btn = match button {
                Button::Left => MouseButton::Left,
                Button::Right => MouseButton::Right,
                Button::Middle => MouseButton::Middle,
                Button::Unknown(code) => MouseButton::Other(code),
            };

            let pos = (event.position.0 as i32, event.position.1 as i32);
            Some(ActionEvent::new_mouse(mouse_btn, pos))
        }

        RdevEventType::MouseMove { x, y } if config.enable_mouse_move => Some(
            ActionEvent::new_mouse(MouseButton::Move, (x as i32, y as i32)),
        ),

        _ => None,
    }
}
