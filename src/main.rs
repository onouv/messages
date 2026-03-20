mod messaging;

use messaging::{ComponentCreatedMessage, ComponentDTO, Message, MessageHeader, Publisher};
use crate::messaging::{ComponentCreatedEvent, ComponentDeletedEvent, ComponentDeletedMessage};

static CREATED: &str = "ComponentCreated";
static DELETED: &str = "ComponentDeleted";

fn main() {
    method_1();
    method_2();
}

fn method_1() {
    println!("METHOD 1: Dedicated Data Types, not useful");
    let comp1 = ComponentDTO::new("100.001", "Door Lock", None);

    let created = ComponentCreatedMessage::new(
        MessageHeader::new("1", CREATED, "-100.001", "process"),
        comp1,
    );

    let deleted =
        ComponentDeletedMessage::new(MessageHeader::new("2", DELETED, "-100.001", "safety"));
    
    println!("{:#?}", created);
    println!("{:#?}", deleted);
}

fn method_2() {
    println!("METHOD 2: Json payloads");
    let view = "safety";
    let component = ComponentDTO::new("-100.001", "Door Lock", None);
    let created = match ComponentCreatedEvent::to_event(component, view) {
        Ok(e) => e,
        Err(e) => {
            println!("ERROR creating ComponentCreatedEvent: {:?}", e);
            return;
        }
    };
    let deleted = ComponentDeletedEvent::to_event("-200.021".to_string(), view);

    let _ = Publisher::publish(created); 
    let _ = Publisher::publish(deleted); 
}

