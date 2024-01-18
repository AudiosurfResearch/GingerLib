use uuid::Uuid;

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub data: Vec<u8>,
}
