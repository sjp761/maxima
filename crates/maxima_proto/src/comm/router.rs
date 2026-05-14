use std::collections::HashMap;

use futures::future::BoxFuture;
use tracing::{debug, info};

use super::proto::{ProtoComponent, ProtoError};

pub type ProtoResult = BoxFuture<'static, Result<Vec<u8>, ProtoError>>;

pub struct RoutingData<'a> {
    pub id: u32,
    pub client_id: u32,
    pub data: &'a [u8],
}

pub struct ProtoRouter {
    components: HashMap<u32, Box<dyn ProtoComponent>>,
}

impl ProtoRouter {
    pub fn builder() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<C: ProtoComponent + 'static>(mut self, component: C) -> Self {
        info!("Registering component {}", component.name());

        self.components.insert(component.id(), Box::new(component));
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
            component.name(),
            component
                .command_name(data.id)
                .unwrap_or(&data.id.to_string()),
            data.client_id,
        );

        component.call(data.id, data.client_id, data.data).await
    }

    pub fn rpc_name(&self, component_id: u32, id: u32) -> String {
        let component = match self.components.get(&component_id) {
            Some(component) => component,
            None => return format!("{component_id}:{id}"),
        };

        let command_name = component
            .command_name(id)
            .map(|x| x.to_owned())
            .unwrap_or(id.to_string());
        format!("{}:{}", component.name(), command_name)
    }
}
