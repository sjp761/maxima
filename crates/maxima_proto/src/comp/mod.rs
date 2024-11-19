pub mod auth;
pub mod users;
pub mod util;

use auth::ClientAuthenticationComponent;
use users::ClientUsersComponent;
use util::ClientUtilitiesComponent;

use crate::proto_client_component_manager;

proto_client_component_manager!(
    Authentication auth,
    Users users,
    Utilities util,
);
