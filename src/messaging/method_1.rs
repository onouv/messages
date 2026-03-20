use serde::{Deserialize, Serialize};

pub trait Message {
    fn msg_id(&self) -> String;
    fn msg_type(&self) -> String;
    fn aggregate_id(&self) -> String;
    fn view_id(&self) -> String;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageHeader {
    msg_id: String,
    msg_type: String,
    aggregate_id: String,
    view_id: String,
    //created_at: DateTime<Utc>
}

impl MessageHeader {
    pub fn new(msg_id: &str, msg_type: &str, aggregate_id: &str, view_id: &str) -> Self {
        Self {
            msg_id: msg_id.to_string(),
            msg_type: msg_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            view_id: view_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentDTO {
    id: String,
    name: String,
    description: Option<String>,
}

impl ComponentDTO {
    pub fn new(id: &str, name: &str, description: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentCreatedMessage {
    header: MessageHeader,
    component: ComponentDTO,
}

impl ComponentCreatedMessage {
    pub fn new(header: MessageHeader, component: ComponentDTO) -> Self {
        Self { header, component }
    }
}

impl Message for ComponentCreatedMessage {
    fn msg_id(&self) -> String {
        self.header.msg_id.clone()
    }
    fn msg_type(&self) -> String {
        self.header.msg_type.clone()
    }
    fn aggregate_id(&self) -> String {
        self.header.aggregate_id.clone()
    }
    fn view_id(&self) -> String {
        self.header.view_id.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentDeletedMessage {
    header: MessageHeader,
}

impl ComponentDeletedMessage {
    pub fn new(header: MessageHeader) -> Self {
        Self { header }
    }
}

impl Message for ComponentDeletedMessage {
    fn msg_id(&self) -> String {
        self.header.msg_id.clone()
    }
    fn msg_type(&self) -> String {
        self.header.msg_type.clone()
    }
    fn aggregate_id(&self) -> String {
        self.header.aggregate_id.clone()
    }
    fn view_id(&self) -> String {
        self.header.view_id.clone()
    }
}
