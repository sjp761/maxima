use crate::{models::user::User, proto_component};

proto_component!(
    Users;

    errors {
        #[error("User Not Found")]
        UserNotFound,
    }

    rpc {
        /// Will throw AuthenticationRequired when not logged in
        local_user(()): User,
    }
);
