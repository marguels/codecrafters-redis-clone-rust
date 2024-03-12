use std::collections::HashMap;

#[derive(Debug)]
pub struct Storage {
    data: HashMap<String, String>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn get(&self, key: String) -> Option<&String> {
        self.data.get(&key)
    }
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(
            key,
            value,
        );
    }
}