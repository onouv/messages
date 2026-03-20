use serde::{Deserialize, Serialize};
use serde_json::*;
use super::ComponentDTO;

#[derive(Serialize)]
pub struct Event {
    event_type: String,
    aggregate_type: String,
    aggregate_id: String,
    view_id: String,
    payload: Option<serde_json::Value>,
}

impl Event {
    pub fn default() -> Self {
        Self {
            event_type: String::new(),
            aggregate_type: String::new(),
            aggregate_id: String::new(),
            view_id: String::new(),
            payload: None 
        }
    }

    pub fn with_event_type(mut self, event_type: &str) -> Self {
        self.event_type = event_type.to_string();
        self
    }
    
    pub fn with_aggregate_type(mut self, aggregate_type: &str) -> Self {
        self.aggregate_type = aggregate_type.to_string();
        self
    }
    
    pub fn with_aggregate_id(mut self, id: &str) -> Self {
        self.aggregate_id = id.to_string();
        self
    }

    pub fn with_view_id(mut self, id: &str) -> Self {
        self.view_id = id.to_string();
        self
    }

    pub fn with_payload<T: Serialize>(mut self, payload: T) -> Result<Self> {
        self.payload = match serde_json::to_value::<T>(payload) {
            Ok(v) => Some(v),
            Err(e) => {
                return Err(e);
            }
        };

        Ok(self)
    }
}


pub struct ComponentCreatedEvent {}

impl ComponentCreatedEvent {
    pub fn to_event(component: ComponentDTO, view_id: &str) -> Result<Event> {
        
        Event::default()
            .with_event_type("created")
            .with_aggregate_type("component")
            .with_aggregate_id(component.id())
            .with_view_id(view_id)
            .with_payload::<ComponentDTO>(component)    
    } 
}


pub struct ComponentDeletedEvent {}

impl ComponentDeletedEvent {
    pub fn to_event(component_id: String, view_id: &str) -> Event {
        Event::default()
            .with_event_type("deleted")
            .with_aggregate_type("component")
            .with_aggregate_id(&component_id)
            .with_view_id(view_id)
    }
}
