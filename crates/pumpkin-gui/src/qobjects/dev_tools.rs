//! Development and CI hooks, driven entirely by environment variables.

// cxx-qt expands into generated glue that does not follow the workspace's lint profile.
#![allow(
    clippy::used_underscore_binding,
    clippy::unnecessary_box_returns,
    clippy::needless_lifetimes,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// `PUMPKIN_GUI_SCREENSHOT`: where to write the PNG. Empty disables the whole mechanism.
        #[qproperty(QString, screenshot_path)]
        /// `PUMPKIN_GUI_SCREENSHOT_DELAY_MS`: how long to let the UI settle first.
        #[qproperty(i32, screenshot_delay_ms)]
        /// `PUMPKIN_GUI_TAB`: which tab to open on, so a screenshot needs no mouse.
        #[qproperty(i32, initial_tab)]
        type DevTools = super::DevToolsRust;
    }
}

use cxx_qt_lib::QString;

pub struct DevToolsRust {
    screenshot_path: QString,
    screenshot_delay_ms: i32,
    initial_tab: i32,
}

impl Default for DevToolsRust {
    fn default() -> Self {
        Self {
            screenshot_path: QString::from(&env_string("PUMPKIN_GUI_SCREENSHOT")),
            // Two sampler ticks by default, so the first frame is not all zeroes.
            screenshot_delay_ms: env_i32("PUMPKIN_GUI_SCREENSHOT_DELAY_MS", 1500),
            initial_tab: env_i32("PUMPKIN_GUI_TAB", 0),
        }
    }
}

fn env_string(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn env_i32(key: &str, fallback: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(fallback)
}
