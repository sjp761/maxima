use crate::{proto_component, proto_struct};

proto_struct!(IdentificationRequest, {
    /// A string identifying the connecting client.
    client_id: String,
    version: String,
});

proto_struct!(IdentificationResponse, {
    client_id: u32,
    server_version: String,
});

proto_component!(
    Utilities;

    errors {
        #[error("Incompatible Server Version")]
        IncompatibleVersion,
    }

    rpc {
        identify(IdentificationRequest): IdentificationResponse,
    }
);
