use std::time::Duration;

use crate::{comm::client::ProtoConnectionManager, comp::{ClientComponentManager, util::IdentificationRequest}};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn setup_client() -> (ProtoConnectionManager, ClientComponentManager) {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with(fmt::Layer::default())
        .init();

    let conn_man = ProtoConnectionManager::new(Duration::from_secs(5));
    let comp_man = ClientComponentManager::new(conn_man.clone());

     let req = IdentificationRequest::builder()
        .client_id("Test".to_owned())
        .version("Test".to_owned())
        .build();

    let _ = comp_man
        .util()
        .identify(req)
        .await
        .expect("Failed to identify");
    
    (conn_man, comp_man)
}
