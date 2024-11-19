use std::{io, sync::Arc};

use bytes::{Buf, BufMut, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tracing::{debug, error, info, warn};

use crate::comm::{
    proto::{ProtoError, ProtoHeader, ProtoPacketType},
    router::{ProtoRouter, RoutingData},
};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub struct ProtoServerPacket {
    pub packet_id: u32,
    pub packet_type: ProtoPacketType,
    pub component: u32,
    pub command: u32,
    pub data: Vec<u8>,
}

pub struct ProtoServer {
    router: Arc<ProtoRouter>,
}

impl ProtoServer {
    pub fn new(router: ProtoRouter) -> Self {
        Self {
            router: Arc::new(router),
        }
    }

    pub async fn serve(&self) -> Result<(), ServerError> {
        let addr = "127.0.0.1:3727".to_string();

        let listener = TcpListener::bind(&addr).await?;
        info!("Listening on: {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;

            let router = self.router.clone();
            tokio::spawn(async move {
                Self::handle_stream(stream, router).await;
            });
        }
    }

    async fn handle_stream(mut stream: TcpStream, router: Arc<ProtoRouter>) {
        let client_id = rand::random::<u32>();

        let mut expected_size: i32 = -1;
        let mut bytes = BytesMut::with_capacity(1024 * 12);

        info!("New client connected: '{client_id}'");

        let (request_tx, mut request_rx) = mpsc::channel(32);

        loop {
            tokio::select! {
                size = stream.read_buf(&mut bytes) => {
                    match size {
                        Ok(0) => {
                            warn!("Client connection '{client_id}' closed");
                            break;
                        },
                        Ok(_) => {
                            loop {
                                debug!("Reading message {} bytes", bytes.len());

                                if bytes.len() < ProtoHeader::SIZE {
                                    break;
                                }

                                if expected_size == -1 {
                                    let mut cloned = bytes.clone().freeze();
                                    let header = ProtoHeader::from(&mut cloned);

                                    expected_size = header.data_size as i32;
                                }

                                let full_expected_size = (expected_size + ProtoHeader::SIZE as i32) as usize;
                                if bytes.len() < full_expected_size {
                                    break;
                                }

                                debug!("Got full message, {} bytes", full_expected_size);

                                // We have the full message now, processing time
                                let mut buf = bytes.clone().freeze().slice(0usize..full_expected_size);

                                bytes.advance(full_expected_size);
                                expected_size = -1;

                                let header = ProtoHeader::from(&mut buf);
                                if header.packet_type == ProtoPacketType::Ping {
                                    request_tx.send(ProtoServerPacket {
                                        packet_id: header.packet_id,
                                        packet_type: ProtoPacketType::PingReply,
                                        component: header.component,
                                        command: header.command,
                                        data: vec![],
                                    }).await.unwrap();
                                    continue;
                                }

                                assert!(buf.remaining() == header.data_size as usize, "Payload size mismatch: {} != {} / {}", buf.remaining(), header.data_size as usize, full_expected_size);

                                debug!("Message: {:?}", header);

                                let router = router.clone();
                                let request_tx = request_tx.clone();

                                tokio::spawn(async move {
                                    match router.call(header.component, RoutingData {
                                        id: header.command,
                                        client_id,
                                        data: &buf
                                    }).await {
                                        Ok(data) => {
                                            request_tx.send(ProtoServerPacket {
                                                packet_id: header.packet_id,
                                                packet_type: ProtoPacketType::Reply,
                                                component: header.component,
                                                command: header.command,
                                                data
                                            }).await.unwrap();
                                        },
                                        Err(err) => {
                                            error!("[{}] RPC from client '{}' Failed: {}", router.rpc_name(header.component, header.command), client_id, err);

                                            let err = match err {
                                                ProtoError::AnyhowError(err) => ProtoError::Internal(err.to_string()),
                                                ProtoError::IoError(err) => ProtoError::Internal(err.to_string()),
                                                _ => err
                                            };

                                            request_tx.send(ProtoServerPacket {
                                                packet_id: header.packet_id,
                                                packet_type: ProtoPacketType::Error,
                                                component: header.component,
                                                command: header.command,
                                                data: serde_json::to_vec(&err).expect("Failed to serialize error")
                                            }).await.unwrap();
                                        },
                                    }
                                });
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {} ({} existing bytes)", e, bytes.len());
                            break;
                        }
                    }
                }
                request = request_rx.recv(), if expected_size == -1 => {
                    if let Some(request) = request {
                        debug!("Sending message {}, {}", request.packet_id, request.data.len() as u32);

                        let mut buf = BytesMut::new();
                        let header = ProtoHeader {
                            data_size: request.data.len() as u32,
                            packet_id: request.packet_id,
                            packet_type: request.packet_type,
                            component: request.component,
                            command: request.command,
                        };

                        header.serialize(&mut buf);
                        buf.put(request.data.as_slice());

                        let mut frozen = buf.freeze();
                        stream.write_buf(&mut frozen).await.unwrap();
                        stream.flush().await.unwrap();
                    }
                },
            }
        }
    }
}
