use maxima_proto::comp::{auth::CheckAuthRequest, util::IdentificationRequest};
use tracing::info;

mod entry;

#[tokio::main]
async fn main() {
    let (_, component_man) = entry::client_setup::create_conn_man();

    let req = IdentificationRequest::builder()
        .client_id("Test".to_owned())
        .version("Test".to_owned())
        .build();

    let _ = component_man.util().identify(req).await.expect("Failed to identify");

    let req = CheckAuthRequest::builder().allow_cached(false).build();
    let res = component_man.auth().check(req).await;
    info!("Logged in?: {:#?}", res.unwrap());
}
