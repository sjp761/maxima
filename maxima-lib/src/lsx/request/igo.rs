use log::info;

use crate::lsx::{
    connection::ConnectionState,
    request::LSXRequestError,
    types::{LSXResponseType, LSXShowIGOWindow},
};

pub async fn handle_show_igo_window_request(
    state: &mut ConnectionState,
    request: LSXShowIGOWindow,
) -> Result<Option<LSXResponseType>, LSXRequestError> {
    info!("Got request to show user {}", request.target_id);

    let maxima = state.maxima().lock().await;
    let data = maxima.player_by_id(&request.target_id.to_string()).await?;

    info!("{:?}", data);
    Ok(None)
}
