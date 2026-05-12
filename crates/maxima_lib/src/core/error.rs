use thiserror::Error;

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
