/// Single item dragged into the list
#[derive(Clone, Debug)]
pub struct DropItem {
    pub display_name: String,
    /// Raw data
    pub data: Vec<u8>,
    pub mime_type: String,
}
