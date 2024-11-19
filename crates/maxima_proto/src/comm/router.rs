use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use tracing::{debug, info};

use super::proto::{ProtoComponent, ProtoError};

pub type ProtoResult = BoxFuture<'static, Result<Vec<u8>, ProtoError>>;

pub struct RoutingData<'a> {
    pub id: u32,
    pub client_id: u32,
    pub data: &'a [u8],
}

type ProtoComponentCallFn = Arc<dyn Fn(RoutingData) -> ProtoResult + Send + Sync>;
type ProtoComponentCommandNameFn = Arc<dyn Fn(u32) -> Option<&'static str> + Send + Sync>;

struct RouterComponent {
    name: &'static str,
    call_fn: ProtoComponentCallFn,
    command_name_fn: ProtoComponentCommandNameFn,
}

impl RouterComponent {
    pub fn new<C: ProtoComponent + 'static>(component: C) -> Self {
        let component = Arc::new(component);
        let component2 = component.clone(); // Is there a better way to do this?

        Self {
            name: C::NAME,
            call_fn: Arc::new(move |data| component.call(data.id, data.client_id, data.data)),
            command_name_fn: Arc::new(move |id| component2.command_name(id)),
        }
    }
}

pub struct ProtoRouter {
    components: HashMap<u32, RouterComponent>,
}

impl ProtoRouter {
    pub fn builder() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<C: ProtoComponent + 'static>(mut self, component: C) -> Self {
        info!("Registering component {}", C::NAME);

        self.components
            .insert(C::ID, RouterComponent::new(component));
        self
    }

    pub(crate) async fn call(
        &self,
        component_id: u32,
        data: RoutingData<'_>,
    ) -> Result<Vec<u8>, ProtoError> {
        let component = match self.components.get(&component_id) {
            Some(component) => component,
            None => return Err(ProtoError::UnknownComponent(component_id)),
        };

        debug!(
            "[{}:{}] Client '{}' is calling RPC",
            component.name,
            (component.command_name_fn)(data.id).unwrap_or(&data.id.to_string()),
            data.client_id,
        );

        (component.call_fn)(data).await
    }

    pub fn _command_name(&self, component_id: u32, id: u32) -> Result<&'static str, ProtoError> {
        let component = match self.components.get(&component_id) {
            Some(component) => component,
            None => return Err(ProtoError::UnknownComponent(component_id)),
        };

        (component.command_name_fn)(id).ok_or(ProtoError::UnknownCommand(component_id, id))
    }

    pub fn rpc_name(&self, component_id: u32, id: u32) -> String {
        let component = match self.components.get(&component_id) {
            Some(component) => component,
            None => return format!("{component_id}:{id}"),
        };

        let command_name = (component.command_name_fn)(id)
            .map(|x| x.to_owned())
            .unwrap_or(id.to_string());
        format!("{}:{}", component.name, command_name)
    }
}
