use std::{
    collections::HashMap,
    error::Error,
    io::{self, ErrorKind},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot, Mutex},
    time,
};
use tracing::{debug, error, info, warn};

use crate::comm::proto::{ProtoHeader, ProtoPacketType};

use super::proto::ProtoError;

const TCP_HOST: &str = "127.0.0.1:3727";

pub struct ClientProtoRequest {
    packet_type: ProtoPacketType,
    id: u32,
    component: u32,
    command: u32,
    payload: Vec<u8>,
    response_tx: Option<oneshot::Sender<Result<Vec<u8>, String>>>,
}

pub trait ProtoNotificationHandler {
    fn handle(&self, payload: Vec<u8>);
}

type ProtoNotificationMap =
    Arc<Mutex<HashMap<(u32, u32), Vec<(usize, Arc<dyn Fn(Vec<u8>) + Send + Sync>)>>>>;

#[derive(Clone)]
pub struct ProtoConnectionManager {
    request_tx: mpsc::Sender<ClientProtoRequest>,
    request_index: Arc<AtomicU32>,
    notification_handlers: ProtoNotificationMap,
    handler_id_counter: Arc<Mutex<usize>>,
}

impl ProtoConnectionManager {
    pub fn new(reconnect_delay: Duration) -> Self {
        let (request_tx, request_rx) = mpsc::channel(32);

        let notification_map = Arc::new(Mutex::new(HashMap::new()));

        let cloned_notif_map = notification_map.clone();
        tokio::spawn(async move {
            ProtoConnectionManager::run(reconnect_delay, request_rx, cloned_notif_map).await;
        });

        let man = Self {
            request_tx: request_tx.clone(),
            request_index: Arc::new(AtomicU32::new(0)),
            notification_handlers: notification_map,
            handler_id_counter: Arc::new(Mutex::new(0)),
        };

        let cloned_man = man.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                cloned_man.send_ping().await.unwrap();
            }
        });

        man
    }

    async fn run(
        reconnect_delay: Duration,
        mut request_rx: mpsc::Receiver<ClientProtoRequest>,
        notification_map: ProtoNotificationMap,
    ) {
        loop {
            let tcp_host =
                std::env::var("MAXIMA_SERVER_HOST").unwrap_or_else(|_| TCP_HOST.to_string());
            match TcpStream::connect(tcp_host).await {
                Ok(stream) => {
                    if let Err(e) = ProtoConnectionManager::handle_stream(
                        stream,
                        &mut request_rx,
                        &notification_map,
                    )
                    .await
                    {
                        error!("Stream error: {}", e);
                        // Reconnection will be attempted after the delay
                    }
                }
                Err(e) => {
                    error!("Failed to connect: {}", e);
                }
            }

            time::sleep(reconnect_delay).await;
        }
    }

    async fn handle_stream(
        mut stream: TcpStream,
        request_rx: &mut mpsc::Receiver<ClientProtoRequest>,
        notification_map: &ProtoNotificationMap,
    ) -> Result<(), Box<dyn Error>> {
        let mut pending_responses: HashMap<u32, oneshot::Sender<Result<Vec<u8>, String>>> =
            HashMap::new();

        let mut expected_size: i32 = -1;
        let mut bytes = BytesMut::with_capacity(1024 * 12);

        info!("Connected");

        loop {
            tokio::select! {
                size = stream.read_buf(&mut bytes) => {
                    match size {
                        Ok(0) => {
                            warn!("Proto connection closed");
                            break;
                        },
                        Ok(_) => {
                            loop {
                                debug!("Reading message {} bytes", bytes.len());

                                if bytes.len() < ProtoHeader::SIZE as usize {
                                    break;
                                }

                                if expected_size == -1 {
                                    let mut cloned = bytes.clone().freeze();
                                    let header = ProtoHeader::from(&mut cloned);

                                    expected_size = (header.data_size as u32) as i32;
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

                                assert!(buf.remaining() == header.data_size as usize, "Payload size mismatch: {} != {} / {}", buf.remaining(), header.data_size as usize, full_expected_size);

                                debug!("Message: {:?}", header);

                                match header.packet_type {
                                    ProtoPacketType::Message => {
                                        unimplemented!("Client should not receive messages")
                                    }
                                    ProtoPacketType::Reply => {
                                        if let Some(tx) =
                                            pending_responses.remove(&header.packet_id)
                                        {
                                            tx.send(Ok(buf.to_vec())).ok();
                                        } else {
                                            println!("Received message with no pending response: {}", header.packet_id);
                                        }
                                    }
                                    ProtoPacketType::Error => {
                                        if let Some(tx) =
                                            pending_responses.remove(&header.packet_id)
                                        {
                                            tx.send(Err(String::from_utf8(buf.to_vec()).expect("Failed to convert server error data to string"))).ok();
                                        } else {
                                            println!("Received message with no pending response: {}", header.packet_id);
                                        }
                                    }
                                    ProtoPacketType::Notification => {
                                        let handlers = notification_map.lock().await;
                                        if let Some(handlers_vec) = handlers.get(&(header.component.clone(), header.command.clone())) {
                                            for (_, handler) in handlers_vec {
                                                handler(buf.to_vec());
                                            }
                                        } else {
                                            debug!("Received notification with no handler [Component: {}, Command: {}]", header.component, header.command);
                                        }
                                    }
                                    ProtoPacketType::PingReply => {}
                                    _ => {
                                        info!("Received non-reply type: {:?} [Component: {}, Command: {}]", header.packet_type, header.component, header.command);
                                    }
                                };
                            }
                        },
                        Err(e) => {
                            println!("Failed to read from proto socket: {} ({} existing bytes)", e, bytes.len());
                            break;
                        },
                    }
                },
                request = request_rx.recv(), if expected_size == -1 => {
                    if let Some(request) = request {
                        debug!("Sending message {}, {}", request.id, request.payload.len() as u32);

                        let mut buf = BytesMut::new();
                        let header = ProtoHeader {
                            data_size: request.payload.len() as u32,
                            packet_id: request.id,
                            packet_type: request.packet_type,
                            component: request.component,
                            command: request.command,
                        };

                        header.serialize(&mut buf);
                        buf.put(request.payload.as_slice());

                        let mut frozen = buf.freeze();
                        stream.write_buf(&mut frozen).await.unwrap();
                        stream.flush().await.unwrap();

                        if let Some(response_tx) = request.response_tx {
                            pending_responses.insert(request.id, response_tx);
                        }
                    }
                },
            }
        }

        info!("Disconnected");
        Ok(())
    }

    pub async fn register_notification_handler(
        &self,
        component: u32,
        command: u32,
        handler: Arc<dyn Fn(Vec<u8>) + Send + Sync>,
    ) -> usize {
        let mut id_counter = self.handler_id_counter.lock().await;
        let id = *id_counter;
        *id_counter += 1;
        self.notification_handlers
            .lock()
            .await
            .entry((component, command))
            .or_insert_with(Vec::new)
            .push((id, handler));
        id
    }

    pub async fn unregister_notification_handler(
        &self,
        component: u32,
        command: u32,
        handler_id: usize,
    ) {
        let mut handlers = self.notification_handlers.lock().await;
        if let Some(handlers_vec) = handlers.get_mut(&(component, command)) {
            handlers_vec.retain(|(id, _)| *id != handler_id);
        }
    }

    pub async fn send_request_raw(
        &self,
        component: u32,
        command: u32,
        message: Vec<u8>,
    ) -> Result<Vec<u8>, ProtoError> {
        let (response_tx, response_rx) = oneshot::channel();

        let index = self.request_index.fetch_add(1, Ordering::SeqCst);

        self.request_tx
            .send(ClientProtoRequest {
                packet_type: ProtoPacketType::Message,
                id: index,
                component,
                command,
                payload: message,
                response_tx: Some(response_tx),
            })
            .await
            .map_err(|x| ProtoError::SendError(x))?;

        match response_rx.await {
            Ok(response) => response.map_err(|x| serde_json::from_str(&x).expect("Failed to deserialize error")),
            Err(err) => Err(ProtoError::IoError(io::Error::new(
                ErrorKind::Other,
                format!("Failed to receive response: {:?}", err),
            ))),
        }
    }

    pub async fn send_request<M, R>(
        &self,
        component: u32,
        command: u32,
        message: M,
    ) -> Result<R, ProtoError>
    where
        M: Serialize,
        R: for<'a> Deserialize<'a>,
    {
        match self
            .send_request_raw(
                component,
                command,
                serde_json::to_vec(&message).expect("Failed to "),
            )
            .await
        {
            Ok(x) => {
                if x.is_empty() {
                    return Err(ProtoError::NoData);
                }

                Ok(serde_json::from_slice(&x).expect("Failed to deserialize response"))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn send_ping(&self) -> Result<(), mpsc::error::SendError<ClientProtoRequest>> {
        let index = self.request_index.fetch_add(1, Ordering::SeqCst);

        self.request_tx
            .send(ClientProtoRequest {
                packet_type: ProtoPacketType::Ping,
                id: index,
                component: 0,
                command: 0,
                payload: Vec::new(),
                response_tx: None,
            })
            .await
    }
}
