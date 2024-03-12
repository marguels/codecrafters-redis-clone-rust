use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Storage {
    data: HashMap<String, ValueWithExpiry>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn get(&self, key: String) -> Option<&String> {
        self.data.get(&key).and_then(|v| {
            if v.is_expired() {
                None
            } else {
                Some(&v.value)
            }
        })
    }
    pub fn set(&mut self, key: String, value: String, expiry_duration: Option<Duration>) {
        self.data.insert(
            key,
            ValueWithExpiry::new(value, expiry_duration),
        );
    }
}

#[derive(Debug)]
struct ValueWithExpiry {
    value: String,
    expiry: Option<Instant>,
}

impl ValueWithExpiry {
    pub fn new(value: String, expiry_duration: Option<Duration>) -> Self {
        ValueWithExpiry {
            value,
            expiry: expiry_duration.map(|d| Instant::now() + d),
        }
    }

    pub fn is_expired(&self) -> bool {
       match self.expiry {
           Some(expiry) => expiry < Instant::now(),
           None => false,
       }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn key_expiration() {
        let mut storage = Storage::new();

        storage.set("foo".to_string(), "bar".to_string(), Some(Duration::from_millis(50)));

        assert_eq!(storage.get("foo".to_string()).is_some(), true);

       
        std::thread::sleep(Duration::from_millis(60));

        assert_eq!(storage.get("foo".to_string()).is_none(), true);
    }
}