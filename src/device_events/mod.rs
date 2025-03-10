//! Devices events listeners.

mod callback;
pub(crate) mod event_loop;

pub use self::callback::*;
use self::event_loop::*;

use {DeviceQuery, KeyEvent};
use DeviceState;

/// All the supported devices events.
pub trait DeviceEvents: DeviceQuery {
    /// Register an on key down event callback.
    fn on_key_down<Callback: Fn(&KeyEvent) -> bool + Sync + Send + 'static>(
        &self,
        callback: Callback,
    ) -> CallbackGuard<Callback>;
    
    /// Register an on key up event callback.
    fn on_key_up<Callback: Fn(&KeyEvent) -> bool + Sync + Send + 'static>(
        &self,
        callback: Callback,
    ) -> CallbackGuard<Callback>;
}

impl DeviceEvents for DeviceState {
    fn on_key_down<Callback: Fn(&KeyEvent) -> bool + Sync + Send + 'static>(
        &self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        EVENT_LOOP
            .lock()
            .expect("Couldn't lock EVENT_LOOP")
            .on_key_down(callback)
    }

    fn on_key_up<Callback: Fn(&KeyEvent) -> bool + Sync + Send + 'static>(
        &self,
        callback: Callback,
    ) -> CallbackGuard<Callback> {
        EVENT_LOOP
            .lock()
            .expect("Couldn't lock EVENT_LOOP")
            .on_key_up(callback)
    }
}
