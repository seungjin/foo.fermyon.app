use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Item {
    #[serde(skip_deserializing)]
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) lastname: String,
}

impl Item {
    pub(crate) fn new(id: i64, name: String, lastname: String) -> Self {
        Self { id, name, lastname }
    }
}
