#[derive(Clone, Debug)]
pub struct CollectionOptions {
    pub dir: String,
    pub ignores: Option<Vec<String>>,
}
