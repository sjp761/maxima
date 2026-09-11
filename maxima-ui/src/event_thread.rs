use egui::Context;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::Duration,
};

use crate::bridge_thread::BackendError;
use log::{error, info};
use maxima::core::{
    LockedMaxima,
    service_layer::{
        SERVICE_REQUEST_GETMYFRIENDS, ServiceFriends, ServiceGetMyFriendsRequestBuilder,
    },
};
use maxima::social::client::{SocialClient, SocialEvent, SocialRequest, UserPresence};

// TODO(headassbtw): integrate this into the enum too (out of scope for the PR i wrote this in)
pub struct EventThreadFriendStatusResponse {
    pub id: String,
    pub presence: UserPresence,
}

pub enum MaximaEventResponse {
    FriendStatusResponse(EventThreadFriendStatusResponse),
}

pub enum MaximaEventRequest {
    SubscribeToFriendPresence,
    ShutdownRequest,
}

pub struct EventThread {}

impl EventThread {
    pub fn new(
        ctx: &Context,
        maxima: LockedMaxima,
        rtm_cmd_listener: UnboundedReceiver<MaximaEventRequest>,
        rtm_responder: UnboundedSender<MaximaEventResponse>,
    ) -> Self {
        let context = ctx.clone();

        tokio::task::spawn(async move {
            match EventThread::run(rtm_cmd_listener, rtm_responder, &context, maxima).await {
                Ok(()) => info!("Event thread shut down cleanly"),
                Err(e) => error!("Event thread error: {e}"),
            }
        });

        Self {}
    }

    async fn run(
        mut rtm_cmd_listener: UnboundedReceiver<MaximaEventRequest>,
        rtm_responder: UnboundedSender<MaximaEventResponse>,
        ctx: &Context,
        maxima_arc: LockedMaxima,
    ) -> Result<(), BackendError> {
        let mut maxima = maxima_arc.lock().await;

        let user = maxima.local_user().await?;
        let player = user.player().as_ref().unwrap();
        let persona_id: Vec<String> = vec![player.psd().to_string()];

        let friends: ServiceFriends = maxima
            .service_layer()
            .request(
                &SERVICE_REQUEST_GETMYFRIENDS,
                ServiceGetMyFriendsRequestBuilder::default()
                    .offset(0)
                    .limit(100)
                    .is_mutual_friends_enabled(false)
                    .build()
                    .unwrap(),
            )
            .await?;

        let rtm = maxima.rtm();
        rtm.login().await?;

        rtm.subscribe().await?;

        let mut social_rx = maxima.social().subscribe().await;

        drop(maxima);

        'outer: loop {
            let mut maxima = maxima_arc.lock().await;
            maxima.rtm().heartbeat().await?;
            drop(maxima);

            if let Ok(event) = social_rx.try_recv() {
                match event {
                    SocialEvent::FriendPresence { id, presence } => {
                        let _ = rtm_responder.send(MaximaEventResponse::FriendStatusResponse(
                            EventThreadFriendStatusResponse { id, presence },
                        ));
                    }
                    SocialEvent::Error(_) => {}
                }
            }

            match rtm_cmd_listener.try_recv() {
                Ok(MaximaEventRequest::SubscribeToFriendPresence) => {}
                Ok(MaximaEventRequest::ShutdownRequest) |
                Err(TryRecvError::Disconnected) => break 'outer Ok(()),
                Err(TryRecvError::Empty) => {}
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
}
