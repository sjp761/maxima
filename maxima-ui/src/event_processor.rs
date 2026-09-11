use crate::{MaximaEguiApp, event_thread};

pub fn frontend_processor(app: &mut MaximaEguiApp, ctx: &egui::Context) {
    puffin::profile_function!();

    while let Ok(result) = app.backend.rtm_listener.try_recv() {
        match result {
            event_thread::MaximaEventResponse::FriendStatusResponse(res) => {
                for friend in &mut app.friends {
                    if friend.id != res.id {
                        continue;
                    }

                    friend.online = res.presence.basic.clone();
                    friend.game = res.presence.game_presence.clone();
                }
                ctx.request_repaint();
            }
        }
    }
}
