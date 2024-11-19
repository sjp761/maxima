use maxima_proto::{
    comm::proto::ProtoRequest,
    comp::auth::{ServerAuthenticationComponent, AuthenticationError, CheckAuthRequest, LoginRequest},
};

use crate::core::auth::storage::LockedAuthStorage;

#[derive(Clone)]
pub struct AuthComponent {
    pub auth_storage: LockedAuthStorage,
}

#[maxima_proto::async_trait]
impl ServerAuthenticationComponent for AuthComponent {
    async fn check(&self, request: ProtoRequest<CheckAuthRequest>) -> Result<bool, AuthenticationError> {
        let req = request.into_inner();
        let _allow_cached = req.allow_cached();

        let mut auth_storage = self.auth_storage.lock().await;
        let logged_in = auth_storage.logged_in().await;

        Ok(logged_in)
    }

    async fn login(&self, _request: ProtoRequest<LoginRequest>) -> Result<(), AuthenticationError> {
        Ok(())
    }
}
