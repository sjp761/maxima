#![feature(type_ascription)]
#![feature(slice_pattern)]
#![feature(string_remove_matches)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use core::{auth::storage::AuthStorage, user_man::UserManager};

use maxima_proto::{
    comm::{router::ProtoRouter, server::ProtoServer},
    comp::{auth::AuthenticationServer, users::UsersServer, util::UtilitiesServer},
};
use comp::{auth::AuthComponent, users::UsersComponent, util::UtilComponent};
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use util::native::maxima_dir;

pub mod content;
pub mod core;
pub mod lsx;
pub mod ooa;
pub mod rtm;
pub mod comp;
pub mod util;

#[cfg(unix)]
pub mod unix;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Maxima only supports the x86_64 architecture due to the use of __cpuid");

#[tokio::main]
async fn main() {
    let log_dir = maxima_dir()
        .expect("Failed to find a suitable home directory. Please specify HOME, XDG_DATA_HOME, or MAXIMA_HOME")
        .join("logs");

    let file_appender = tracing_appender::rolling::daily(log_dir, "maxima_server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let mut default_layer = fmt::Layer::default();

    // Waiting on #15701 <https://github.com/rust-lang/rust/issues/15701> to
    // cook this out in release
    if cfg!(debug_assertions) {
        default_layer = default_layer
            .with_file(true)
            .with_line_number(true)
            .with_target(false);
    }

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with(default_layer)
        .with(
            fmt::Layer::default()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    info!(
        "Initializing MAXIMA Server {} ({})...",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    );

    let auth_storage = match AuthStorage::load() {
        Ok(storage) => storage,
        Err(err) => {
            error!("Failed to load auth storage, using default: {err}");
            AuthStorage::new()
        }
    };

    let user_man = UserManager::new(auth_storage.clone());

    let router = ProtoRouter::builder()
        .add_component(AuthenticationServer::new(AuthComponent {
            auth_storage: auth_storage.clone(),
        }))
        .add_component(UsersServer::new(UsersComponent {
            auth_storage: auth_storage.clone(),
            user_man,
        }))
        .add_component(UtilitiesServer::new(UtilComponent { auth_storage }));

    ProtoServer::new(router).serve().await.unwrap();
}
