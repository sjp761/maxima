use crate::util::registry::check_registry_validity;
use tracing::warn;
use crate::core::error::BackgroundServiceControlError;
use crate::util::registry::set_up_registry;

pub fn register_service() -> Result<(), BackgroundServiceControlError> {
    Ok(())
}

pub unsafe fn init_service_security() -> Result<(), BackgroundServiceControlError> {
    Ok(())
}

pub fn is_service_valid() -> Result<bool, BackgroundServiceControlError> {
    Ok(true)
}

pub fn is_service_running() -> Result<bool, BackgroundServiceControlError> {
    Ok(true)
}

pub async fn start_service() -> Result<(), BackgroundServiceControlError> {
    Ok(())
}

pub async fn stop_service() -> Result<(), BackgroundServiceControlError> {
    Ok(())
}

pub fn register_service_user() -> Result<(), BackgroundServiceControlError> {
    Ok(())
}

#[cfg(not(windows))]
pub async fn service_setup() -> Result<(), BackgroundServiceControlError> {
   
    if let Err(err) = check_registry_validity() {
        warn!("{}, fixing...", err);
        set_up_registry().unwrap();
    }

    Ok(())
}