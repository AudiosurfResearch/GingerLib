use uuid::Uuid;

#[derive(Debug)]
pub struct Tag {
    name: String,
    data: Vec<u8>,
}

#[derive(Debug)]
/// Struct representing a channel.
/// They act as nodes in visual scripting.
/// They may store any arbitrary data in the form of "tags".
pub struct Channel {
    guid: Uuid,
    name: String,
}
