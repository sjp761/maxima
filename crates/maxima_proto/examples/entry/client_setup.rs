use std::time::Duration;

use maxima_proto::{comm::client::ProtoConnectionManager, comp::ClientComponentManager};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn create_conn_man() -> (ProtoConnectionManager, ClientComponentManager) {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with(fmt::Layer::default())
        .init();

    let conn_man = ProtoConnectionManager::new(Duration::from_secs(5));
    let comp_man = ClientComponentManager::new(conn_man.clone());

    (conn_man, comp_man)
}
