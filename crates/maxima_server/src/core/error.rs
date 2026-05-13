use maxima::util::native::NativeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheRetrievalError {
    #[error(transparent)]
    Native(#[from] NativeError),
    #[error(transparent)]
    ServiceLayer(#[from] crate::core::service_layer::ServiceLayerError),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("incapable of pulling {0} from cache")]
    Incapable(String),
}
