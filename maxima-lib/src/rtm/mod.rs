pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/eadp.rtm.rs"));
}

pub mod connection;
pub mod client;
