pub mod github;
pub mod native;
pub mod registry;

#[cfg(windows)]
pub mod dll_injector;

#[cfg(windows)]
pub mod service {
    include!("service_win.rs");
}

#[cfg(unix)]
#[allow(dead_code)]
pub mod service {
    include!("service_nix.rs");
}
