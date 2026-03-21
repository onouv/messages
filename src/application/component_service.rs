use anyhow::Result;
use uuid::Uuid;

use crate::adapters::downstream::db::ComponentRepository;
use crate::messaging::{ComponentCreatedEvent, ComponentDTO};

pub struct ComponentService {
    repo: ComponentRepository,
    view_id: String,
}

impl ComponentService {
    pub fn new(repo: ComponentRepository, view_id: &str) -> Self {
        Self {
            repo,
            view_id: view_id.to_string(),
        }
    }

    pub async fn create_component(&self, id: &str, name: &str) -> Result<ComponentDTO> {
        let component = ComponentDTO::new(id, name, None);
        let event = ComponentCreatedEvent::to_event(component.clone(), &self.view_id)?;
        let event_payload = serde_json::to_value(&event)?;

        self.repo
            .create(&component, Uuid::new_v4(), "created", &self.view_id, event_payload)
            .await?;

        Ok(component)
    }
}
