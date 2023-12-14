pub mod native;
pub mod log;
pub mod registry;
pub mod simple_crypto;
pub mod github;

#[cfg(target_os = "windows")]
pub mod service {
    include!("service_win.rs");
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub mod service {
    include!("service_nix.rs");
}