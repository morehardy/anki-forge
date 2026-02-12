#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMeta {
    pub collection_filename: String,
}

impl Default for PackageMeta {
    fn default() -> Self {
        Self {
            collection_filename: String::from("collection.anki2"),
        }
    }
}
