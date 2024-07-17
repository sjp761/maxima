use anyhow::Result;
use log::{debug, info};

use crate::{
    lsx::{
        connection::LockedConnectionState,
        types::{
            LSXErrorSuccess, LSXFriend, LSXFriendState, LSXGetPresence, LSXGetPresenceResponse,
            LSXGetProfile, LSXGetProfileResponse, LSXImage, LSXPresence, LSXQueryFriends,
            LSXQueryFriendsResponse, LSXQueryImage, LSXQueryImageResponse, LSXQueryPresence,
            LSXQueryPresenceResponse, LSXResponseType, LSXSetPresence,
        },
    },
    make_lsx_handler_response,
    rtm::client::{BasicPresence, RichPresenceBuilder},
    util::native::platform_path,
};

pub async fn handle_profile_request(
    state: LockedConnectionState,
    _: LSXGetProfile,
) -> Result<Option<LSXResponseType>> {
    let arc = state.write().await.maxima_arc();
    let maxima = arc.lock().await;

    let user = maxima.local_user().await?;
    let path = platform_path(maxima.avatar_image(&user.id(), 208, 208).await?);

    let player = user.player().as_ref().unwrap();
    let name = player.unique_name();
    debug!("Got profile for {} {:?}", &name, path);

    make_lsx_handler_response!(Response, GetProfileResponse, {
       attr_Persona: name.to_owned(),
       attr_SubscriberLevel: 0,
       attr_CommerceCurrency: "USD".to_string(),
       attr_IsTrialSubscriber: false,
       attr_Country: "US".to_string(),
       attr_UserId: user.id().parse::<u64>()?,
       attr_GeoCountry: "US".to_string(),
       attr_AvatarId: path.to_str().unwrap().to_string(),
       attr_IsSubscriber: false,
       attr_IsSteamSubscriber: false,
       attr_PersonaId: player.psd().parse::<u64>()?,
       attr_IsUnderAge: false,
       attr_UserIndex: 0,
    })
}

pub async fn handle_presence_request(
    _: LockedConnectionState,
    _: LSXGetPresence,
) -> Result<Option<LSXResponseType>> {
    make_lsx_handler_response!(Response, GetPresenceResponse, {
       attr_UserId: 1005663144213,
       attr_Presence: LSXPresence::Ingame,
       attr_Title: None,
       attr_TitleId: None,
       attr_MultiplayerId: None,
       attr_RichPresence: None,
       attr_GamePresence: None,
       attr_SessionId: None,
       attr_Group: None,
       attr_GroupId: None,
    })
}

pub async fn handle_set_presence_request(
    state: LockedConnectionState,
    request: LSXSetPresence,
) -> Result<Option<LSXResponseType>> {
    info!(
        "Setting Presence to {:?}: {}",
        request.attr_Presence,
        request
            .attr_RichPresence
            .to_owned()
            .or(Some("Unknown".to_string()))
            .unwrap()
    );

    let arc = state.write().await.maxima_arc();
    let mut maxima = arc.lock().await;

    let playing = maxima.playing().as_ref().unwrap();
    if playing.mode().is_online_offline() {
        return make_lsx_handler_response!(Response, ErrorSuccess, { attr_Code: 0, attr_Description: String::new() });
    }

    let offer = playing.offer().as_ref().unwrap().offer();
    let offer_id = offer.offer_id().to_owned();
    let name = offer.display_name().to_owned();

    if let Some(presence) = request.attr_RichPresence {
        maxima
            .rtm()
            .set_presence(
                BasicPresence::Online,
                &format!("{}: {}", name, presence),
                &offer_id,
            )
            .await?;
    }

    make_lsx_handler_response!(Response, ErrorSuccess, { attr_Code: 0, attr_Description: String::new() })
}

pub async fn handle_query_presence_request(
    state: LockedConnectionState,
    request: LSXQueryPresence,
) -> Result<Option<LSXResponseType>> {
    let mut friends = Vec::new();

    let mut state = state.write().await;
    let mut maxima = state.maxima().await;
    let presence_store = maxima.rtm().presence_store().lock().await;

    for user in request.Users {
        let presence = presence_store.get(&user.to_string());
        if presence.is_none() {
            continue;
        }

        let presence = presence.unwrap();

        let game = if let Some(game) = presence.game() {
            game.to_owned()
        } else {
            String::new()
        };

        friends.push(LSXFriend {
            attr_TitleId: "".to_string(),
            attr_MultiplayerId: "".to_string(),
            attr_Persona: "------".to_string(),
            attr_RichPresence: presence.status().to_string(),
            attr_GamePresence: game,
            attr_Title: "".to_string(),
            attr_UserId: user,
            attr_PersonaId: "0".to_string(),
            attr_AvatarId: "".to_string(),
            attr_Group: "".to_string(),
            attr_GroupId: "".to_string(),
            attr_Presence: LSXPresence::Ingame,
            attr_State: LSXFriendState::None,
        });
    }

    make_lsx_handler_response!(Response, QueryPresenceResponse, { friend: friends })
}

pub async fn handle_query_friends_request(
    state: LockedConnectionState,
    _: LSXQueryFriends,
) -> Result<Option<LSXResponseType>> {
    let mut state = state.write().await;
    let mut maxima = state.maxima().await;

    let friends = maxima.friends(0).await?;
    let presence_store = maxima.rtm().presence_store().lock().await;

    let mut lsx_friends = Vec::new();
    for ele in friends {
        let mut presence = presence_store.get(ele.id());
        if presence.is_none() {
            presence = Some(
                RichPresenceBuilder::default()
                    .basic(BasicPresence::Offline)
                    .status(String::new())
                    .game(None)
                    .build()?,
            );
        }

        let presence = presence.unwrap();

        let mut lsx_presence = match presence.basic() {
            BasicPresence::Unknown => LSXPresence::Unknown,
            BasicPresence::Offline => LSXPresence::Offline,
            BasicPresence::Dnd => LSXPresence::Busy,
            BasicPresence::Away => LSXPresence::Idle,
            BasicPresence::Online => LSXPresence::Online,
        };

        let game = if let Some(game) = presence.game() {
            game.to_owned()
        } else {
            String::new()
        };

        if !game.is_empty() {
            lsx_presence = LSXPresence::Ingame;
        }

        lsx_friends.push(LSXFriend {
            attr_TitleId: "".to_string(),
            attr_MultiplayerId: "".to_string(),
            attr_Persona: ele.unique_name().to_string(),
            attr_RichPresence: presence.status().to_string(),
            attr_GamePresence: game,
            attr_Title: "".to_string(),
            attr_UserId: ele.id().parse()?,
            attr_PersonaId: ele.pd().parse()?,
            attr_AvatarId: "".to_string(),
            attr_Group: "".to_string(),
            attr_GroupId: "".to_string(),
            attr_Presence: lsx_presence,
            attr_State: LSXFriendState::None,
        });
    }

    make_lsx_handler_response!(Response, QueryFriendsResponse, { friend: lsx_friends })
}

pub async fn handle_query_image_request(
    state: LockedConnectionState,
    request: LSXQueryImage,
) -> Result<Option<LSXResponseType>> {
    let parts = request.attr_ImageId.split(":").collect::<Vec<_>>();

    let arc = state.write().await.maxima_arc();
    let maxima = arc.lock().await;

    let path = maxima
        .avatar_image(parts[1], request.attr_Width, request.attr_Height)
        .await?;

    let mut images = Vec::new();

    // TODO Download and populate images
    images.push(LSXImage {
        attr_ImageId: request.attr_ImageId,
        attr_Width: request.attr_Width,
        attr_Height: request.attr_Height,
        attr_ResourcePath: path.to_str().unwrap().to_string(),
    });

    make_lsx_handler_response!(Response, QueryImageResponse, { attr_Result: 1, image: images, })
}
