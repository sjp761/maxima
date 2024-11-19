use std::{sync::Arc, time::Duration};

use anyhow::{bail, Result};
use maxima_proto::models::user::User;
use moka::sync::Cache;
use tokio::sync::Mutex;

use super::{
    auth::storage::LockedAuthStorage,
    service_layer::{
        ServiceGetUserPlayerRequest, ServiceLayerClient, ServiceUser, SERVICE_REQUEST_GETUSERPLAYER,
    },
};

fn service_to_proto_user(user: &ServiceUser) -> User {
    let player = user.player().clone().unwrap_or_default();

    User::builder()
        .account_id(user.id().to_owned())
        .persona_id(player.psd().to_owned())
        .display_name(player.display_name().to_owned())
        .unique_name(player.unique_name().to_owned())
        .nickname(player.nickname().to_owned())
        .build()
}

pub struct UserManager {
    client: ServiceLayerClient,
    cached_users: Cache<String, User>,
}

pub type LockedUserManager = Arc<Mutex<UserManager>>;

impl UserManager {
    pub fn new(auth_storage: LockedAuthStorage) -> LockedUserManager {
        let s = Self {
            client: ServiceLayerClient::new(auth_storage),
            cached_users: Cache::builder()
                .max_capacity(128)
                .time_to_live(Duration::from_secs(30 * 60))
                .time_to_idle(Duration::from_secs(5 * 60))
                .build(),
        };

        Arc::new(Mutex::new(s))
    }

    pub async fn local_user(&self) -> Result<User> {
        if !self.cached_users.contains_key("0") {
            let user: ServiceUser = self
                .client
                .request(
                    SERVICE_REQUEST_GETUSERPLAYER,
                    ServiceGetUserPlayerRequest {},
                )
                .await?;

            self.cached_users
                .insert("0".to_owned(), service_to_proto_user(&user));
        }

        match self.cached_users.get("0") {
            Some(user) => Ok(user),
            None => bail!("No user"),
        }
    }
}
