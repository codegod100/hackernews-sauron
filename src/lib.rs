#![deny(warnings)]
#![deny(unused_extern_crates)]

// Common modules (previously from common crate)
pub mod api;
pub mod types;

// App modules (previously from client crate)
pub use app::{App, Msg};
use sauron::prelude::*;
pub use sauron;

mod app;
pub mod util;

/// The serialized_state is optionally supplied for server-side rendering hydration.
/// For pure client-side applications, an empty string can be passed to start with App::default().
/// The app will then initialize with default state and begin client-side routing.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub async fn main(serialized_state: String) {
    #[cfg(feature = "wasm-bindgen")]
    {
        console_log::init_with_level(log::Level::Trace).ok();
        console_error_panic_hook::set_once();
    }

    let app = match serde_json::from_str::<App>(&serialized_state) {
        Ok(app_state) => app_state,
        Err(e) => {
            log::warn!("error: {}", e);
            App::default()
        }
    };
    Program::replace_mount(app, &sauron::dom::util::body());
}