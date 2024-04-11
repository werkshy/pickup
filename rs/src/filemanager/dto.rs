#[derive(Clone, Debug)]
pub struct CollectionLocation {
    pub category: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc: Option<String>,
    pub track: Option<String>,
}
