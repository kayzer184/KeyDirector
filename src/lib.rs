//! A simple library for querying mouse and keyboard state without requiring
//! an active window. Currently works in Windows, Linux, and macOS.
//!
//! ```no_run
//! use device_query::{DeviceQuery, DeviceState, MouseState, Keycode};
//!
//! let device_state = DeviceState::new();
//!
//! let mouse: MouseState = device_state.get_mouse();
//! println!("Current Mouse Coordinates: {:?}", mouse.coords);
//!
//! let keys: Vec<Keycode> = device_state.get_keys();
//! println!("Is A pressed? {}", keys.contains(&Keycode::A));
//! ```
//!
//! It's also possible to listen for events.
//! ```no_run
//!  use device_query::{DeviceEvents, DeviceState};
//!
//!  let device_state = DeviceState::new();
//!
//!  let _guard = device_state.on_mouse_move(|position| {
//!     println!("Mouse position: {:#?}", position);
//!  });
//!  let _guard = device_state.on_mouse_down(|button| {
//!     println!("Mouse button down: {:#?}", button);
//!  });
//!  let _guard = device_state.on_mouse_up(|button| {
//!     println!("Mouse button up: {:#?}", button);
//!  });
//!  let _guard = device_state.on_key_down(|key| {
//!     println!("Keyboard key down: {:#?}", key);
//!  });
//!  let _guard = device_state.on_key_up(|key| {
//!     println!("Keyboard key up: {:#?}", key);
//!  });
//!
//!  loop {}
//! ```

#[macro_use]
extern crate lazy_static;
extern crate serde;

#[cfg(target_os = "windows")]
extern crate windows;

#[cfg(target_os = "macos")]
extern crate core_foundation;
extern crate macos_accessibility_client;
extern crate cocoa;
extern crate core_graphics;
extern crate objc;

pub mod device_events;
pub mod device_query;
pub mod device_state;
pub mod keymap;

pub use device_events::*;
pub use device_query::*;
pub use device_state::*;
pub use keymap::*;
