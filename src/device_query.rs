//! Query functions.

use crate::DeviceState;
use crate::KeyEvent;
use crate::device_events::event_loop::EVENT_LOOP;
use crate::CallbackGuard;

/// Trait to get the state of the supported devices.
pub trait DeviceQuery {
    /// Get Keyboard state.
    fn get_keys(&self) -> Vec<KeyEvent>;
    
    /// Subscribe to key events
    fn subscribe_keys<F>(&self, callback: F) -> CallbackGuard<F>
    where
        F: Fn(Vec<KeyEvent>) + Send + Sync + 'static;
}

impl DeviceQuery for DeviceState {
    /// Query for all keys that are currently pressed down.
    fn get_keys(&self) -> Vec<KeyEvent> {
        self.query_keymap()
    }
    
    fn subscribe_keys<F>(&self, callback: F) -> CallbackGuard<F>
    where
        F: Fn(Vec<KeyEvent>) + Send + Sync + 'static,
    {
        EVENT_LOOP
            .lock()
            .expect("Couldn't lock EVENT_LOOP")
            .on_keys(callback)
    }
}   