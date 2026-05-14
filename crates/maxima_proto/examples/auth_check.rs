use maxima_proto::comp::{auth::CheckAuthRequest, util::IdentificationRequest};
use tracing::info;

use maxima_proto::entry::client_setup;
#[tokio::main]
async fn main() {
    let (_, component_man) = client_setup::setup_client().await;

   

    let req = CheckAuthRequest::builder().allow_cached(false).build();
    let res = component_man.auth().check(req).await;
    info!("Logged in?: {:#?}", res.unwrap());
}
