use maxima_proto::{
    comm::proto::{ProtoError, ProtoRequest},
    comp::library::{LibraryError, ServerLibraryComponent},
    models::library::Game,
};

use crate::core::{auth::storage::LockedAuthStorage, user_man::LockedUserManager};

#[derive(Clone)]
pub struct LibraryComponent {
    pub auth_storage: LockedAuthStorage,
    pub user_man: LockedUserManager,
}

#[maxima_proto::async_trait]
impl ServerLibraryComponent for LibraryComponent {
    async fn games(&self, _request: ProtoRequest<()>) -> Result<Vec<Game>, LibraryError> {
        let user_man = self.user_man.lock().await;

        Err(LibraryError::Proto(ProtoError::NoData))
    }
}
