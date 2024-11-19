use maxima_proto::{
    comm::proto::{ProtoError, ProtoRequest},
    comp::users::{ServerUsersComponent, UsersError},
    models::user::User,
};

use crate::core::{auth::storage::LockedAuthStorage, user_man::LockedUserManager};

#[derive(Clone)]
pub struct UsersComponent {
    pub auth_storage: LockedAuthStorage,
    pub user_man: LockedUserManager,
}

#[maxima_proto::async_trait]
impl ServerUsersComponent for UsersComponent {
    async fn local_user(&self, _request: ProtoRequest<()>) -> Result<User, UsersError> {
        let user_man = self.user_man.lock().await;

        let res = match user_man.local_user().await {
            Ok(user) => Ok(user),
            Err(err) => {
                // Blah.
                if err.to_string().contains("UNAUTHENTICATED") {
                    Err(UsersError::Proto(ProtoError::Unauthenticated))
                } else {
                    Err(err)?
                }
            }
        };

        res
    }
}
