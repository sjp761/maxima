use maxima_proto::comp::{auth::LoginRequest, util::IdentificationRequest};
use tracing::info;

use maxima_proto::entry::client_setup;

#[tokio::main]
async fn main() {
    let (_, component_man) = client_setup::setup_client().await;

    let req = LoginRequest::builder().build();
    let res = component_man.auth().login(req).await;
}
