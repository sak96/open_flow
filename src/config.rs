// Configuration structure
pub struct Config {
    pub screenshot_throttle_ms: u64,
    pub enable_mouse_move: bool,
    pub debug_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            screenshot_throttle_ms: 100,
            enable_mouse_move: true,
            debug_mode: false,
        }
    }
}
