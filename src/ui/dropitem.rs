/// Single item dragged into the list
#[derive(Clone, Debug)]
pub struct DropItem {
    pub display_name: String,
    pub data: String,
    pub mime_type: String,
}
