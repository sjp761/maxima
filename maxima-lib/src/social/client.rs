use super::eadp::{
    common::v1::{DevicePlatformId, PlayerNetworkId, ProductId},
    social::presence::v1::{
        presence_service_client::PresenceServiceClient, presence_update::PropertiesEnt,
        ClientInfo, ConnectToPresenceSessionRequest, ConnectToPresenceSessionResponse,
        CreatePresenceSessionRequest, PresenceSessionToken,
        PresenceUpdate, SubscribeToFriendsPresenceRequest,
    },
};
use super::SocialError;
use crate::core::auth::storage::LockedAuthStorage;
use derive_getters::Getters;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;

use crate::lsx::types::LSXPresence;
use crate::social::eadp::social::presence::v1::{
    value::Value::{IntegerValue, StringValue},
    PartialUpdatePresenceSessionRequest, PartialUpdatePresenceSessionResponse,
};
use log::{error, info};
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Status};
use webpki_roots::TLS_SERVER_ROOTS;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UserPresenceBasic {
    Unknown,
    Offline,
    Online,
    Away,
}

impl Into<LSXPresence> for UserPresenceBasic {
    fn into(self) -> LSXPresence {
        match self {
            UserPresenceBasic::Offline => LSXPresence::Offline,
            UserPresenceBasic::Online => LSXPresence::Online,
            UserPresenceBasic::Away => LSXPresence::Idle,
            _ => LSXPresence::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserPresence {
    pub offer_id: Option<String>,
    pub multiplayer_id: Option<String>,
    pub rich_presence: Option<String>,
    pub game_title: Option<String>,
    pub game_presence: Option<String>,
    pub basic: UserPresenceBasic,

    pub group_name: Option<String>,
    pub group_id: Option<String>,
    pub group_public: Option<bool>,
    pub joinable: Option<bool>,
    pub joinable_invite_only: Option<bool>,
}

impl Default for UserPresence {
    fn default() -> Self {
        Self {
            offer_id: None,
            multiplayer_id: None,
            rich_presence: None,
            game_title: None,
            game_presence: None,
            basic: UserPresenceBasic::Unknown,

            group_name: None,
            group_id: None,
            group_public: None,
            joinable: None,
            joinable_invite_only: None,
        }
    }
}

impl UserPresence {
    pub fn online() -> Self {
        Self {
            basic: UserPresenceBasic::Online,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub enum SocialEvent {
    FriendPresence { id: String, presence: UserPresence },
    Error(SocialError),
}

pub enum SocialRequest {
    UpdatePresence(UserPresence),
}

enum ListenerToSender {
    Token(PresenceSessionToken),
}

#[derive(Getters)]
pub struct SocialClient {
    pub tx: Sender<SocialRequest>,
    /// Backlog of presence updates to send when subscribed
    backlog: Arc<Mutex<Vec<SocialEvent>>>,
    /// Subscribers to presence updates
    senders: Arc<Mutex<Vec<Sender<SocialEvent>>>>,
    /// All currently known friend presences
    pub presence_store: Arc<Mutex<HashMap<String, UserPresence>>>,
}

impl SocialClient {
    pub fn new(auth: LockedAuthStorage) -> Self {
        let auth1 = auth.clone();
        let (tx, social_rx) = std::sync::mpsc::channel::<SocialRequest>();
        let (social_tx, rx) = std::sync::mpsc::channel::<SocialEvent>();
        let social_tx1 = social_tx.clone();
        let (listener_tx, sender_rx) = std::sync::mpsc::channel::<ListenerToSender>();

        let senders = Arc::new(Mutex::new(Vec::new()));
        let senders_burn = senders.clone();

        let backlog = Arc::new(Mutex::new(Vec::new()));
        let backlog_burn = backlog.clone();

        let presence_store = Arc::new(Mutex::new(HashMap::new()));
        let presence_store_burn = presence_store.clone();

        tokio::task::spawn(async move {
            let fallback_tx = social_tx1.clone();
            match SocialClient::run_connection(
                auth1,
                senders_burn,
                backlog_burn,
                presence_store_burn,
                listener_tx,
                rx,
            )
            .await
            {
                Ok(_) => (),
                Err(e) => {
                    error!("Social client error: {}", e);
                    let _ = fallback_tx.send(SocialEvent::Error(e));
                }
            }
            info!("Social connection (listener) closed");
        });

        tokio::task::spawn(async move {
            let fallback_tx = social_tx.clone();
            match SocialClient::run_requests(auth, sender_rx, social_rx).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Social client error: {}", e);
                    let _ = fallback_tx.send(SocialEvent::Error(e));
                }
            }
            info!("Social connection (sender) closed");
        });

        SocialClient {
            tx,
            backlog,
            senders,
            presence_store,
        }
    }

    // TODO(headassbtw): replace this with a normal SocialRequest once the server is polled and not awaited?
    /// Receive social system updates as they come in
    pub async fn subscribe(&mut self) -> Receiver<SocialEvent> {
        let (a, b) = std::sync::mpsc::channel::<SocialEvent>();
        for event in self.backlog.lock().await.iter() {
            let _ = a.send(event.clone());
        }
        self.senders.lock().await.push(a);
        b
    }

    async fn run_requests(
        auth: LockedAuthStorage,
        token_rx: Receiver<ListenerToSender>,
        rx: Receiver<SocialRequest>,
    ) -> Result<(), SocialError> {
        let token = {
            let auth = match auth.lock().await.access_token().await.unwrap() {
                Some(auth) => auth,
                None => return Err(SocialError::NoAccessToken)
            };
            format!("Bearer {}", auth)
        };
        let config = ClientTlsConfig::default().trust_anchors(TLS_SERVER_ROOTS.to_owned());
        let channel = Channel::from_static("https://api.k.social.ea.com")
            .tls_config(config)
            .unwrap()
            .connect()
            .await
            .unwrap();
        let mut client =
            PresenceServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("authorization", token.parse().unwrap());
                Ok(req)
            });

        let mut session_token: Option<PresenceSessionToken> = None;
        'waiter: loop {
            match token_rx.try_recv() {
                Ok(ListenerToSender::Token(token)) => {
                    session_token = Some(token);
                    break 'waiter;
                }
                Err(TryRecvError::Disconnected) => break 'waiter return Ok(()),
                Err(TryRecvError::Empty) => {}
            }
        }

        'outer: loop {
            match token_rx.try_recv() {
                Ok(ListenerToSender::Token(token)) => {
                    session_token = Some(token);
                }
                _ => {}
            }
            match rx.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    break 'outer Ok(());
                }
                Ok(SocialRequest::UpdatePresence(presence)) => {
                    info!("updating social presence");
                    let mut presence_props: Vec<PropertiesEnt> = Vec::new();

                    if let Some(offer_id) = presence.offer_id {
                        presence_props.push(PropertiesEnt::string("ea_app.productId", offer_id));
                    }

                    if let Some(game_title) = &presence.game_title {
                        presence_props.push(PropertiesEnt::string("ea_app.gameTitle", game_title.clone()));
                    }

                    if let Some(rich_presence) = presence.rich_presence {
                        presence_props.push(PropertiesEnt::string("ea_app.richPresence", rich_presence));
                    }

                    if let Some(game_presence) = presence.game_presence {
                        presence_props.push(PropertiesEnt::string("ea_app.presenceStatus", game_presence));
                    } else {
                        if let Some(game_title) = presence.game_title {
                            presence_props.push(PropertiesEnt::string("ea_app.presenceStatus", game_title));
                        }
                    }

                    if let Some(joinable) = presence.joinable {
                        presence_props.push(PropertiesEnt::integer("ea_app.isJoinable", joinable.into()));
                    }

                    if let Some(joinable_invite_only) = presence.joinable_invite_only {
                        presence_props.push(PropertiesEnt::integer("ea_app.isJoinableInviteOnly", joinable_invite_only.into()));
                    }

                    if let Some(multiplayer_id) = presence.multiplayer_id {
                        presence_props.push(PropertiesEnt::string("ea_app.multiplayerId", multiplayer_id));
                    }

                    if let Some(group_id) = presence.group_id {
                        presence_props.push(PropertiesEnt::string("ea_app.groupGuid", group_id));
                    }

                    if let Some(group_name) = presence.group_name {
                        presence_props.push(PropertiesEnt::string("ea_app.groupName", group_name));
                    }

                    if let Some(group_public) = presence.group_public {
                        presence_props.push(PropertiesEnt::integer("ea_app.groupIsPublic", group_public.into()));
                    }

                    presence_props.push(PropertiesEnt::integer("ea_app.presenceIsInvisible", 0));
                    presence_props.push(PropertiesEnt::integer(
                        "ea_app.presenceAvailability",
                        match presence.basic {
                            UserPresenceBasic::Online => -1,
                            UserPresenceBasic::Away => -2,
                            _ => 0,
                        },
                    ));


                    let upd = PresenceUpdate {
                        appear_offline: false,
                        properties: presence_props,
                    };
                    let req = PartialUpdatePresenceSessionRequest {
                        presence_session_token: session_token.clone(),
                        presence_update: Some(upd.clone()),
                    };
                    client.partial_update_presence_session(req).await?;
                }
                _ => {}
            }
        }
    }

    /// For running the actual "heartbeat"
    /// due to technical reasons, this must be run separately and in parallel to `run_requests`
    async fn run_connection(
        auth: LockedAuthStorage,
        senders: Arc<Mutex<Vec<Sender<SocialEvent>>>>,
        backlog: Arc<Mutex<Vec<SocialEvent>>>,
        presence_store: Arc<Mutex<HashMap<String, UserPresence>>>,
        tx: Sender<ListenerToSender>,
        rx: Receiver<SocialEvent>, // this function doesn't need this but i'm keeping it here for shutdown reasons
    ) -> Result<(), SocialError> {
        let token = {
            let auth = match auth.lock().await.access_token().await.unwrap() {
                Some(auth) => auth,
                None => return Err(SocialError::NoAccessToken)
            };
            format!("Bearer {}", auth)
        };
        let config = ClientTlsConfig::default().trust_anchors(TLS_SERVER_ROOTS.to_owned());
        // TODO(headassbtw): less `.unwrap()`s for TLS stuff
        let channel = Channel::from_static("https://api.k.social.ea.com")
            .tls_config(config)
            .unwrap()
            .connect()
            .await
            .unwrap();
        let mut client =
            PresenceServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("authorization", token.parse().unwrap());
                Ok(req)
            });

        let request = tonic::Request::new(CreatePresenceSessionRequest {
            client_info: Some(ClientInfo {
                player_network_id: Some(PlayerNetworkId::ea()),
                product_id: Some(ProductId::juno()),
                device_platform_id: Some(DevicePlatformId::pc()),
                locale: "en_US".to_owned(),
            }),
        });

        let res = client.create_presence_session(request).await?;
        let token = res.into_inner().presence_session_token;
        let res = client
            .subscribe_to_friends_presence(SubscribeToFriendsPresenceRequest {
                presence_session_token: token.clone(),
            })
            .await?;

        let _ = tx.send(ListenerToSender::Token(token.clone().unwrap_or_default()));

        let req = ConnectToPresenceSessionRequest {
            presence_session_token: token.clone(),
            presence: Vec::new(),
        };

        'guh: loop {
            let mut resp = client
                .connect_to_presence_session(req.clone())
                .await?
                .into_inner();
            while let Some(update) = resp.next().await {
                match Self::handle_update(
                    update,
                    senders.clone(),
                    backlog.clone(),
                    presence_store.clone(),
                )
                .await
                {
                    Ok(_) => (),
                    Err(e) => error!("Social client error: {}", e),
                }
            }

            match rx.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    drop(resp);
                    break 'guh;
                }
                _ => {}
            }
            info!("Reconnecting to social client");
        }

        info!("Social client disconnected");
        Ok(())
    }

    async fn handle_update(
        update: Result<ConnectToPresenceSessionResponse, Status>,
        senders: Arc<Mutex<Vec<Sender<SocialEvent>>>>,
        backlog: Arc<Mutex<Vec<SocialEvent>>>,
        presence_store: Arc<Mutex<HashMap<String, UserPresence>>>,
    ) -> Result<(), SocialError> {
        if let Ok(presence) = update {
            let presence = presence.presence_notification.unwrap();
            let id = presence.player_id.unwrap().id;
            let mut user_presence = UserPresence::default();
            if presence.player_online {
                let presence = presence.session_notification.unwrap();
                info!("{:?}", presence);

                for prop in presence.properties {
                    let value = prop.value.unwrap().value.unwrap();
                    match prop.key.as_str() {
                        "ea_app.presenceAvailability" => {
                            if let IntegerValue(val) = value {
                                user_presence.basic = match val {
                                    -1 => UserPresenceBasic::Online,
                                    -2 => UserPresenceBasic::Away,
                                    _ => UserPresenceBasic::Unknown,
                                };
                            }
                        }
                        "ea_app.productId" => {
                            if let StringValue(val) = value {
                                user_presence.offer_id = Some(val);
                            }
                        } // Offer ID
                        "ea_app.gameTitle" => {
                            if let StringValue(val) = value {
                                user_presence.game_title = Some(val);
                            }
                        } // "Battlefield 4"
                        "ea_app.richPresence" => {
                            if let StringValue(val) = value {
                                user_presence.rich_presence = Some(val);
                            }
                        } // "In the menus"
                        "ea_app.presenceStatus" => {
                            if let StringValue(val) = value {
                                user_presence.game_presence = Some(val);
                            }
                        } // "Battlefield 4 In the menus"
                        "ea_app.isJoinable" => {
                            if let IntegerValue(val) = value {
                                user_presence.joinable = Some(val > 0);
                            }
                        } // unknown, safe to guess
                        "ea_app.isJoinableInviteOnly" => {
                            if let IntegerValue(val) = value {
                                user_presence.joinable_invite_only = Some(val > 0);
                            }
                        } // unknown, safe to guess
                        "ea_app.multiplayerId" => {
                            if let StringValue(val) = value {
                                user_presence.multiplayer_id = Some(val);
                            }
                        } // "1002645",
                        "ea_app.groupGuid" => {
                            if let StringValue(value) = value {
                                user_presence.group_id = Some(value);
                            }
                        } // unknown
                        "ea_app.groupName" => {
                            if let StringValue(val) = value {
                                user_presence.group_name = Some(val);
                            }
                        } // unknown
                        "ea_app.groupIsPublic" => {
                            if let IntegerValue(val) = value {
                                user_presence.group_public = Some(val > 0);
                            }
                        } // unknown, safe to guess
                        "ea_app.presenceIsInvisible" => {} // this is always present, never used, and never honored.
                        "ea_app.gamePresence" => {}        // appears to be a Base64 string
                        "ea_app.gameSessionString" => {}   // unknown, probably a string
                        unhandled => {
                            error!("unhandled presence property `{} : {:?}`", unhandled, value);
                        }
                    }
                }
            }

            presence_store
                .lock()
                .await
                .insert(id.clone(), user_presence.clone());

            let ev = SocialEvent::FriendPresence {
                id,
                presence: user_presence,
            };
            backlog.lock().await.push(ev.clone());
            for tx in senders.lock().await.iter() {
                let _ = tx.send(ev.clone());
            }
        }

        Ok(())
    }
}
