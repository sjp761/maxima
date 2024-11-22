use crate::{models::library::Game, proto_component};

proto_component!(
    Library;

    errors {
        #[error("Offer not found")]
        OfferNotFound,
        #[error("Cannot launch game as it is not installed")]
        LaunchGameNotInstalled,
        #[error("Game path must be specified when launching in OnlineOffline mode")]
        LaunchGamePathRequired,
        #[error("Content ID was specified as an offer ID when launching in OnlineOffline mode")]
        LaunchContentIdRequired,
    }

    rpc {
        games(()): Vec<Game>,
    }
);
