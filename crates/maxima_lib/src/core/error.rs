use thiserror::Error;

use crate::util::native;

#[derive(Error, Debug)]
pub enum BackgroundServiceClientError {
    #[error(transparent)]
    Native(#[from] crate::util::native::NativeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Registry(#[from] crate::util::registry::RegistryError),
    #[cfg(windows)]
    #[error(transparent)]
    Injection(#[from] crate::util::dll_injector::InjectionError),

    #[error("request failed: `{0}`")]
    Request(String),
    #[error("attempted to inject into invalid process")]
    InvalidInjectionTarget,
}

#[derive(thiserror::Error, Debug)]
pub enum BackgroundServiceControlError {
    #[error(transparent)]
    Native(#[from] native::NativeError),
    #[cfg(windows)]
    #[error(transparent)]
    WindowsService(#[from] windows_service::Error),
    #[error(transparent)]
    Nul(#[from] prost::alloc::ffi::NulError),
    #[cfg(windows)]
    #[error(transparent)]
    WidestringContainsNul(#[from] widestring::error::ContainsNul<u16>),

    #[error("failed to find service when configuring security")]
    Absent,
    #[error("failed to set service security attributes: `{0}`")]
    SecurityAttributes(std::io::Error),
    #[error("unable to convert security descriptor to string: `{0}`")]
    SecurityDescriptorToString(std::io::Error),
    #[error("unable to query service object security: `{0}`")]
    ServiceObjectSecurity(std::io::Error),
    #[error("unable to convert SDDL string to security descriptor: `{0}`")]
    StringToSecurityDescriptor(std::io::Error),
}
