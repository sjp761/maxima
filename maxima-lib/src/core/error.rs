use std::path::PathBuf;

use snafu::Snafu;
use tokio::io;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to read configuration from {}", path.display()))]
    ReadConfiguration { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to write result to {}", path.display()))]
    WriteResult { source: io::Error, path: PathBuf },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
