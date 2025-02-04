//! Query functions.

use crate::DeviceState;
use crate::{KeyEvent, MouseState};

/// Trait to get the state of the supported devices.
pub trait DeviceQuery {
    /// Get MouseState.
    fn get_mouse(&self) -> MouseState;

    /// Get Keyboard state.
    fn get_keys(&self) -> Vec<KeyEvent>;
}

impl DeviceQuery for DeviceState {
    /// Query for the current mouse position and mouse button device_state.
    fn get_mouse(&self) -> MouseState {
        self.query_pointer()
    }

    /// Query for all keys that are currently pressed down.
    fn get_keys(&self) -> Vec<KeyEvent> {
        self.query_keymap()
    }
}