use tracing::info;

use maxima_proto::{
    comm::proto::ProtoRequest,
    comp::util::{IdentificationRequest, IdentificationResponse, ServerUtilitiesComponent, UtilitiesError},
};

use crate::core::auth::storage::LockedAuthStorage;

#[derive(Clone)]
pub struct UtilComponent {
    pub auth_storage: LockedAuthStorage,
}

#[maxima_proto::async_trait]
impl ServerUtilitiesComponent for UtilComponent {
    async fn identify(
        &self,
        request: ProtoRequest<IdentificationRequest>,
    ) -> Result<IdentificationResponse, UtilitiesError> {
        let client_id = request.client_id();

        let req = request.into_inner();
        info!("Client '{}' identified: {}", client_id, req.client_id());

        Ok(IdentificationResponse::builder()
            .client_id(client_id)
            .server_version(env!("CARGO_PKG_VERSION").to_owned())
            .build())
    }
}
