mod linux;
mod macos;
mod target;
mod windows;

use target::Target;

pub fn local_target() -> Target {
    match std::env::consts::OS {
        "macos" => {
        Target { host: "Darwin".into(), target_str: "arm64-apple-darwin20.3.0".into() }
        },
        _ => {
            panic!("Platform {} unsupported, please check the issue tracker.", std::env::consts::OS);
        }
    }
}

// #[cfg(target_os = "macos")]
// pub mod macos;
// #[cfg(target_os = "linux")]
// static DEFAULT_PATH: &str = "path0";
// #[cfg(target_os = "windows")]
// static DEFAULT_PATH: &str = "path1";
