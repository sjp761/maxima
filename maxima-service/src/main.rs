use anyhow::Result;

#[cfg(windows)]
use service::start_service;

#[cfg(windows)]
mod service;

#[cfg(windows)]
fn main() -> Result<()> {
    start_service()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    
}