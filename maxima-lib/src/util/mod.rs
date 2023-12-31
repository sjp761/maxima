pub mod native;
pub mod log;
pub mod registry;
pub mod simple_crypto;
pub mod github;

#[cfg(windows)]
pub mod service {
    include!("service_win.rs");
}

#[cfg(unix)]
#[allow(dead_code)]
pub mod service {
    include!("service_nix.rs");
}