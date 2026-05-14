use tracing::info;

use maxima_proto::entry::client_setup;


#[tokio::main]
async fn main() {
    let (_, comp_man) = client_setup::setup_client().await;

    let req = ();
    let res = comp_man.users().local_user(req).await;
    info!("User: {:#?}", res.unwrap());
}
