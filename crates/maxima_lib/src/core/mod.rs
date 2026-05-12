#[cfg(target_os = "windows")]
mod background_service_win;

#[cfg(target_os = "linux")]
mod background_service_nix;

pub mod background_service {
    #[cfg(target_os = "windows")]
    pub use super::background_service_win::*;

    #[cfg(target_os = "linux")]
    pub use super::background_service_nix::*;
}

pub mod error;
