use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

const FIRST_OID: i32 = 0x10000000;

#[derive(Debug)]
pub struct IdFactory {
    next_id: AtomicI32,
    reusable_ids: Mutex<Vec<i32>>,
}

impl IdFactory {
    pub fn instance() -> Arc<Self> {
        static INSTANCE: std::sync::OnceLock<Arc<IdFactory>> = std::sync::OnceLock::new();
        INSTANCE
            .get_or_init(|| {
                Arc::new(IdFactory {
                    next_id: AtomicI32::new(FIRST_OID),
                    reusable_ids: Mutex::new(Vec::new()),
                })
            })
            .clone()
    }

    pub fn get_next_id(&self) -> i32 {
        let mut reusable = self.reusable_ids.lock().unwrap();
        if let Some(id) = reusable.pop() {
            return id;
        }
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn release_id(&self, id: i32) {
        let mut reusable = self.reusable_ids.lock().unwrap();
        reusable.push(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {
        let factory = IdFactory::instance();
        let id1 = factory.get_next_id();
        let id2 = factory.get_next_id();
        assert_ne!(id1, id2);
        assert!(id1 >= FIRST_OID);
        assert!(id2 > id1);
    }

    #[test]
    fn test_reuse() {
        let factory = IdFactory::instance();
        let id1 = factory.get_next_id();
        factory.release_id(id1);
        let id2 = factory.get_next_id();
        assert_eq!(id1, id2);
    }
}
