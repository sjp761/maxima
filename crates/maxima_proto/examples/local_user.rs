use maxima_proto::comp::util::IdentificationRequest;
use tracing::info;

mod entry;

#[tokio::main]
async fn main() {
    let (_, comp_man) = entry::client_setup::create_conn_man();

    let req = IdentificationRequest::builder()
        .client_id("Test".to_owned())
        .version("Test".to_owned())
        .build();

    let _ = comp_man.util().identify(req).await.expect("Failed to identify");

    let req = ();
    let res = comp_man.users().local_user(req).await;
    info!("User: {:#?}", res.unwrap());
}
