use thiserror::Error;

pub mod eadp {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("eadp.common.v1");
            impl PlayerNetworkId {
                pub fn ea() -> Self {
                    Self {
                        id: "EA".to_owned(),
                    }
                }
            }
            impl ProductId {
                pub fn juno() -> Self {
                    Self {
                        id: "01eb04f5-ad3f-7a1c-ff56-892bb262b1a4".to_owned(),
                    }
                }
            }
            impl DevicePlatformId {
                pub fn pc() -> Self {
                    Self {
                        id: "PC".to_owned(),
                    }
                }
            }
        }
        pub mod v2 {
            tonic::include_proto!("eadp.common.v2");
        }
    }
    pub mod social {
        pub mod presence {
            pub mod v1 {
                use crate::social::eadp::social::presence::v1::value::Value::{IntegerValue, StringValue, };

                tonic::include_proto!("eadp.social.presence.v1");

                impl presence_update::PropertiesEnt {
                    pub fn integer(key: &str, val: i64) -> Self {
                        Self {
                            key: key.to_string(),
                            value: Some(Value {
                                value: Some(IntegerValue(val))
                            }),
                        }
                    }

                    pub fn string(key: &str, val: String) -> Self {
                        Self {
                            key: key.to_string(),
                            value: Some(Value {
                                value: Some(StringValue(val))
                            }),
                        }
                    }
                }
            }
        }
    }
}

pub mod client;

#[derive(Error, Debug, Clone)]
pub enum SocialError {
    #[error(transparent)]
    Recv(#[from] std::sync::mpsc::RecvError),
    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),
    #[error("No access token")]
    NoAccessToken,
}
