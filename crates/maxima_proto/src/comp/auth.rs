use crate::{proto_component, proto_struct};

proto_struct!(LoginRequest, {
});

proto_struct!(CheckAuthRequest, {
    /// If false, the current login will be checked
    /// against EA servers for validity. If true,
    /// only the current login token's expiry will
    /// be checked
    allow_cached: bool,
});

proto_component!(
    Authentication;

    rpc {
        check(CheckAuthRequest): bool,
        login(LoginRequest): (),
    }
);
