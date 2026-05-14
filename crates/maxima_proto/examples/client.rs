use maxima_proto::comp::util::IdentificationRequest;
use tracing::info;

use maxima_proto::entry::client_setup;

#[tokio::main]
async fn main() {
    let (_, component_man) = client_setup::setup_client().await;

    let req = IdentificationRequest::builder()
        .client_id("Test".to_owned())
        .version("Test".to_owned())
        .build();

    let response = component_man.util().identify(req).await;
    info!("Response: {:#?}", response.unwrap());
}
