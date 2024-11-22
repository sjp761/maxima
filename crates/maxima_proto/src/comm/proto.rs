use std::io;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::{client::ClientProtoRequest, router::ProtoResult};

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtoError {
    /// This is messy because ProtoError is inside the component error
    /// enums, which makes this sort of cyclic. This is needed for the
    /// upstreaming of errors from the component to router layer
    #[error("Component error: {0}")]
    ComponentError(String),
    /// Internal is the client representation of the forwarded
    /// server error types. An error log will occur on the server
    /// whenever an Internal error is sent to the client
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Anyhow error: {0}")]
    #[serde(skip)]
    AnyhowError(#[from] anyhow::Error),
    #[error("IO error: {0}")]
    #[serde(skip)]
    IoError(#[from] io::Error),
    #[error("Send error: {0}")]
    #[serde(skip)]
    SendError(#[from] mpsc::error::SendError<ClientProtoRequest>),
    #[error("Server did not send any data")]
    NoData,
    #[error("Unknown component: {0}")]
    UnknownComponent(u32),
    #[error("Unknown command {1} in component {0}")]
    UnknownCommand(u32, u32),
    #[error("The requested command requires authentication")]
    Unauthenticated,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ProtoPacketType {
    Message,
    Reply,
    Error,
    Notification,
    Ping,
    PingReply,
}

#[derive(Debug)]
#[repr(C)]
pub struct ProtoHeader {
    pub data_size: u32,
    pub packet_id: u32,
    pub packet_type: ProtoPacketType,
    pub component: u32,
    pub command: u32,
}

impl ProtoHeader {
    pub const SIZE: usize = 4 + 4 + 1 + 4 + 4;

    pub fn from(buf: &mut Bytes) -> Self {
        Self {
            data_size: buf.get_u32(),
            packet_id: buf.get_u32(),
            packet_type: match buf.get_u8() {
                0 => ProtoPacketType::Message,
                1 => ProtoPacketType::Reply,
                2 => ProtoPacketType::Error,
                3 => ProtoPacketType::Notification,
                4 => ProtoPacketType::Ping,
                5 => ProtoPacketType::PingReply,
                val => panic!("Invalid message type {val}"),
            },
            component: buf.get_u32(),
            command: buf.get_u32(),
        }
    }

    pub fn serialize(&self, buf: &mut BytesMut) {
        buf.put_u32(self.data_size);
        buf.put_u32(self.packet_id);
        buf.put_u8(self.packet_type as u8);
        buf.put_u32(self.component);
        buf.put_u32(self.command);
    }
}

pub trait ProtoComponent: Send + Sync {
    const ID: u32;
    const NAME: &'static str;

    fn command_name(&self, id: u32) -> Option<&'static str>;
    fn call(&self, id: u32, client_id: u32, data: &[u8]) -> ProtoResult;
}

pub struct ProtoRequest<T> {
    inner: T,
    client_id: u32,
}

impl<T> ProtoRequest<T> {
    pub fn new(inner: T, client_id: u32) -> Self {
        Self { inner, client_id }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn client_id(&self) -> u32 {
        self.client_id
    }
}

#[macro_export]
macro_rules! proto_component {
    (
        $component_name:ident;
        $(errors {
            $(
                $(#[$error_field_attr:meta])*
                $error_name:ident
            ),* $(,)?
        })?
        $(rpc {
            $(
                $(#[$command_field_attr:meta])*
                $command_name:ident($command_var:ty): $command_return_type:ty
            ),* $(,)?
        })?
    ) => {
        paste::paste! {
            const [<PROTO_COMPONENT_ $component_name:upper _ID>]: u32 =
                const_fnv1a_hash::fnv1a_hash_32(stringify!($component_name).as_bytes(), None);

            #[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
            pub enum [<$component_name Error>] {
                #[error("Proto error: {0}")]
                Proto(#[from] crate::comm::proto::ProtoError),
                $(
                    $(
                        $(#[$error_field_attr])*
                        $error_name
                    ),*
                )?
            }

            impl From<std::io::Error> for [<$component_name Error>] {
                fn from(value: std::io::Error) -> Self {
                    Self::Proto(crate::comm::proto::ProtoError::IoError(value))
                }
            }

            impl From<anyhow::Error> for [<$component_name Error>] {
                fn from(value: anyhow::Error) -> Self {
                    Self::Proto(crate::comm::proto::ProtoError::AnyhowError(value))
                }
            }

            $(
                $(
                    const [<PROTO_COMPONENT_ $component_name:upper _COMMAND_ $command_name:upper _ID>]: u32 =
                        const_fnv1a_hash::fnv1a_hash_32(stringify!($command_name).as_bytes(), None);
                )*

                #[async_trait::async_trait]
                pub trait [<Server $component_name Component>]: Clone + Send + Sync + 'static {
                    const ID: u32 = [<PROTO_COMPONENT_ $component_name:upper _ID>];
                    const NAME: &'static str = stringify!($component_name);

                    $(
                        const [<ID_ $command_name:upper>]: u32 = [<PROTO_COMPONENT_ $component_name:upper _COMMAND_ $command_name:upper _ID>];
                        
                        $(#[$command_field_attr])*
                        async fn $command_name(&self, request: crate::comm::proto::ProtoRequest<$command_var>) -> Result<$command_return_type, [<$component_name Error>]>;
                    )*
                }

                #[derive(Debug, Clone)]
                pub struct [<$component_name Server>]<T: [<Server $component_name Component>]> {
                    inner: std::sync::Arc<T>,
                }

                impl<T: [<Server $component_name Component>]> [<$component_name Server>]<T> {
                    pub fn new(inner: T) -> Self {
                        Self { inner: std::sync::Arc::new(inner) }
                    }
                }

                impl<T: [<Server $component_name Component>]> crate::comm::proto::ProtoComponent for [<$component_name Server>]<T> {
                    const ID: u32 = T::ID;
                    const NAME: &'static str = T::NAME;

                    fn command_name(&self, id: u32) -> Option<&'static str> {
                        // We unfortunately can't use a match here because of weird pattern requirements for const variables.
                        $(
                            if id == T::[<ID_ $command_name:upper>] { return Some(stringify!($command_name)); }
                        )*
                        None
                    }

                    fn call(&self, id: u32, client_id: u32, data: &[u8]) -> crate::comm::router::ProtoResult {
                        let inner = self.inner.clone();

                        $(
                            // See above note.
                            if id == T::[<ID_ $command_name:upper>] {
                                let request_data = serde_json::from_slice(data).expect("Failed to deserialize request");
                                let request = crate::comm::proto::ProtoRequest::new(request_data, client_id);
                                return Box::pin(async move {
                                    let result = <T as [<Server $component_name Component>]>::$command_name(&inner, request).await;
                                    result.map(|x| serde_json::to_vec(&x).expect("Failed to serialize response")).map_err(|x| {
                                        #[allow(irrefutable_let_patterns)]
                                        if let [<$component_name Error>]::Proto(proto) = x {
                                            return proto;
                                        }

                                        crate::comm::proto::ProtoError::ComponentError(serde_json::to_string(&x).expect("Failed to serialize error"))
                                    })
                                });
                            }
                        )*

                        Box::pin(async move {
                            Err(crate::comm::proto::ProtoError::UnknownCommand(Self::ID, id))
                        })
                    }
                }

                #[derive(Clone)]
                pub struct [<Client $component_name Component>] {
                    pub conn_man: crate::comm::client::ProtoConnectionManager,
                }

                impl [<Client $component_name Component>] {
                    pub fn new(conn_man: crate::comm::client::ProtoConnectionManager) -> Self {
                        Self { conn_man }
                    }

                    $(
                        $(#[$command_field_attr])*
                        pub async fn $command_name(&self, request: $command_var) -> Result<$command_return_type, [<$component_name Error>]> {
                            match self.conn_man.send_request(
                                [<PROTO_COMPONENT_ $component_name:upper _ID>],
                                [<PROTO_COMPONENT_ $component_name:upper _COMMAND_ $command_name:upper _ID>],
                                request
                            ).await {
                                Ok(res) => Ok(res),
                                Err(err) => {
                                    if let crate::comm::proto::ProtoError::ComponentError(data) = err {
                                        return Err(serde_json::from_str(&data).expect("Failed to deserialize error"));
                                    }

                                    Err([<$component_name Error>]::Proto(err))
                                }
                            }
                        }
                    )*
                }
            )?
        }
    };
}

#[macro_export]
macro_rules! proto_client_component_manager {
    (
        $($component_name:ident $snake_name:ident),* $(,)?
    ) => {
        paste::paste! {
            #[derive(Clone)]
            pub struct ClientComponentManager {
                $(
                    $snake_name: [<Client $component_name Component>],
                )*
            }

            impl ClientComponentManager {
                pub fn new(conn_man: crate::comm::client::ProtoConnectionManager) -> Self {
                    Self {
                        $(
                            $snake_name: [<Client $component_name Component>]::new(conn_man.clone()),
                        )*
                    }
                }

                $(
                    pub fn $snake_name(&self) -> & [<Client $component_name Component>] {
                        &self.$snake_name
                    }
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! proto_struct {
    ($name:ident, { $($(#[$meta:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, typed_builder::TypedBuilder, derive_getters::Getters)]
        pub struct $name {
            $(
                $(#[$meta])*
                $field: $ty,
            )*
        }
    };
}

