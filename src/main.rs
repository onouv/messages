mod messaging;
use messaging::{ComponentCreatedMessage, ComponentDTO, Message, MessageHeader};

use crate::messaging::ComponentDeletedMessage;

static CREATED: &str = "ComponentCreated";
static DELETED: &str = "ComponentDeleted";

fn main() {
    let comp1 = ComponentDTO::new("100.001", "Door Lock", None);
    let comp2 = ComponentDTO::new("100.002", "Door Terminal", None);

    let created = ComponentCreatedMessage::new(
        MessageHeader::new("1", CREATED, "100.001", "process"),
        comp1,
    );

    let deleted =
        ComponentDeletedMessage::new(MessageHeader::new("2", DELETED, "100.001", "safety"));
    print_msg(&created);
    print_msg(&deleted);
}

fn print_msg(msg: &impl Message) {
    println!(
        "(msg_id: {}, msg_type: {}, aggregate_id: {}, view_id: {})",
        msg.msg_id(),
        msg.msg_type(),
        msg.aggregate_id(),
        msg.view_id()
    );
}
