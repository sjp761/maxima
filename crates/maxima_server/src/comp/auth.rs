use maxima_proto::{
    comm::proto::ProtoRequest,
    comp::auth::{
        AuthenticationError, CheckAuthRequest, LoginRequest, ServerAuthenticationComponent,
    },
};

use crate::core::auth::{
    context::AuthContext, login::begin_oauth_login_flow, nucleus_token_exchange,
    storage::LockedAuthStorage,
};

#[derive(Clone)]
pub struct AuthComponent {
    pub auth_storage: LockedAuthStorage,
}

#[maxima_proto::async_trait]
impl ServerAuthenticationComponent for AuthComponent {
    async fn check(
        &self,
        request: ProtoRequest<CheckAuthRequest>,
    ) -> Result<bool, AuthenticationError> {
        let req = request.into_inner();
        let _allow_cached = req.allow_cached();

        let mut auth_storage = self.auth_storage.lock().await;
        let logged_in = auth_storage.logged_in().await;

        Ok(logged_in.unwrap()) // Placeholder for now
    }

    async fn login(&self, _request: ProtoRequest<LoginRequest>) -> Result<(), AuthenticationError> {
        let mut auth_context = AuthContext::new().unwrap();
        begin_oauth_login_flow(&mut auth_context).await.unwrap();
        let token_res = nucleus_token_exchange(&auth_context).await.unwrap();
        let mut storage = self.auth_storage.lock().await;
        let _ = storage.add_account(&token_res).await;
        Ok(())
    }

    async fn access_token(&self, _request: ProtoRequest<()>) -> Result<String, AuthenticationError> {
        let mut storage = self.auth_storage.lock().await;
        let token = storage.access_token_or_err().await.unwrap();
        Ok(token)
    }
}
