//! Query functions.

use crate::DeviceState;
use crate::KeyEvent;

/// Trait to get the state of the supported devices.
pub trait DeviceQuery {
    /// Get Keyboard state.
    fn get_keys(&self) -> Vec<KeyEvent>;
}

impl DeviceQuery for DeviceState {
    /// Query for all keys that are currently pressed down.
    fn get_keys(&self) -> Vec<KeyEvent> {
        self.query_keymap()
    }
}