use std::time::SystemTime;

use anyhow::Error;

#[derive(Debug)]
pub struct InMemoryRecord {
    pub value: String,
    pub last_update_time: SystemTime,
    pub expire_milli: Option<u128>,
}

impl InMemoryRecord {
    pub fn new(value: String, expire_milli: Option<u128>) -> Self {
        InMemoryRecord {
            value,
            // No need for UTC. This is just an internal date.
            last_update_time: SystemTime::now(),
            expire_milli,
        }
    }

    pub fn has_expired(&self) -> Result<bool, Error> {
        Ok(self.expire_milli.is_some()
            && SystemTime::now()
                .duration_since(self.last_update_time)?
                .as_millis()
                > self.expire_milli.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryRecord;

    use std::{thread, time::Duration};

    #[test]
    fn has_expired_passes() -> Result<(), anyhow::Error> {
        let expires = InMemoryRecord::new("".to_owned(), Some(1));
        let does_not_expire = InMemoryRecord::new("".to_owned(), Some(3));

        thread::sleep(Duration::from_millis(2));

        assert_eq!(expires.has_expired()?, true);
        assert_eq!(does_not_expire.has_expired()?, false);

        Ok(())
    }
}
