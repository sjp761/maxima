use tokio::net::TcpListener;

use log::{info, warn};

use crate::lsx::connection::LSXConnectionError;
use crate::{core::LockedMaxima, lsx::connection::Connection};
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[derive(Error, Debug)]
pub enum LSXServerError {
    #[error(transparent)]
    Conn(#[from] LSXConnectionError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub async fn start_server(port: u16, maxima: LockedMaxima) -> Result<(), LSXServerError> {
    let addr = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(err) => return Err(LSXServerError::Io(err)),
        };

        info!("New LSX connection: {:?}", addr);
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let mut conn = match Connection::new(maxima.clone(), socket, tx).await {
            Ok(c) => c,
            Err(err) => {
                warn!("Failed to establish LSX connection: {}", err);
                continue;
            }
        };

        conn.queue_challenge().await?;

        let mut maxima = maxima.lock().await;
        maxima.inc_connected_lsx();
        maxima.set_player_started();
        drop(maxima);

        tokio::spawn(async move {
            if let Err(err) = handle_client(conn, rx).await {
                warn!("LSX connection error: {}", err);
            }
        });
    }
}

pub async fn handle_client(
    mut conn: Connection,
    mut rx: Receiver<String>,
) -> Result<(), LSXServerError> {
    loop {
        tokio::select! {
            res = conn.listen() => {
                match res {
                    Ok(_) => continue,
                    Err(err) => {
                        warn!("LSX connection error: {}", err);
                        return Err(LSXServerError::Conn(err));
                    }
                }
            },
            Some(msg) = rx.recv() => {
                if let Err(err) = conn.write_message(msg).await {
                    warn!("LSX connection error: {}", err);
                    return Err(LSXServerError::Conn(err));
                }
            },
        }
    }
}
