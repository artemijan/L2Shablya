use dashmap::DashSet;
use log::warn;
use std::collections::HashSet;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

const FIRST_OID: i32 = 0x1000_0000;

#[derive(Debug)]
pub struct IdFactory {
    next_id: AtomicI32,
    reusable_ids: Mutex<HashSet<i32>>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(Arc<i32>);

impl ObjectId {
    #[must_use]
    pub fn new(id: i32) -> Self {
        Self(Arc::new(id))
    }
    #[must_use]
    pub fn value(&self) -> i32 {
        *self.0
    }
}

impl From<ObjectId> for i32 {
    fn from(id: ObjectId) -> i32 {
        *id.0
    }
}

impl PartialEq<i32> for ObjectId {
    fn eq(&self, other: &i32) -> bool {
        *self.0 == *other
    }
}
impl PartialEq<ObjectId> for i32 {
    fn eq(&self, other: &ObjectId) -> bool {
        *self == *other.0
    }
}
impl PartialOrd<ObjectId> for i32 {
    fn partial_cmp(&self, other: &ObjectId) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&*other.0)
    }
}
impl Drop for ObjectId {
    fn drop(&mut self) {
        // Only release if this is the last Arc
        if Arc::strong_count(&self.0) == 1 {
            IdFactory::instance().release_id(*self.0);
        }
    }
}
impl PartialOrd<i32> for ObjectId {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        (*self.0).partial_cmp(other)
    }
}

impl IdFactory {
    pub fn instance() -> Arc<Self> {
        static INSTANCE: std::sync::OnceLock<Arc<IdFactory>> = std::sync::OnceLock::new();
        INSTANCE
            .get_or_init(|| {
                Arc::new(IdFactory {
                    next_id: AtomicI32::new(FIRST_OID),
                    reusable_ids: Mutex::new(HashSet::new()),
                })
            })
            .clone()
    }

    pub fn get_next_id(&self) -> ObjectId {
        let mut set = self.get_locked_state();
        let Some(&id) = set.iter().next() else {
            return ObjectId::new(self.next_id.fetch_add(1, Ordering::SeqCst));
        };
        set.remove(&id);
        ObjectId::new(id)
    }

    pub fn release_id(&self, id: impl Into<i32>) {
        let val = id.into();
        if !self.get_locked_state().insert(val) {
            warn!("Trying to release already released id: {val}");
        }
    }
    fn get_locked_state(&self) -> std::sync::MutexGuard<'_, HashSet<i32>> {
        self.reusable_ids
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl IdFactory {
        fn reset_for_tests(&self) {
            let mut set = self.get_locked_state();
            set.clear();
            self.next_id.store(FIRST_OID, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_allocation() {
        let factory = IdFactory::instance();
        factory.reset_for_tests();
        let id1 = factory.get_next_id();
        let id2 = factory.get_next_id();
        assert_ne!(id1, id2);
        assert!(id1 >= ObjectId::new(FIRST_OID));
        assert!(id2 > id1);
    }

    #[test]
    fn test_reuse() {
        let factory = IdFactory::instance();
        factory.reset_for_tests();
        let id_copy: i32;
        {
            let id1 = factory.get_next_id();
            id_copy = id1.clone().into();
            drop(id1);
        } //drop id1
        let id2 = factory.get_next_id();
        assert_eq!(id2, id_copy);
    }

    #[test]
    fn test_cloned() {
        let factory = IdFactory::instance();
        factory.reset_for_tests();
        let id_copy: i32;
        {
            let id1 = factory.get_next_id();
            let id2 = id1.clone();
            assert_eq!(id1, id2);
            drop(id1);
            //id1 is dropped, but the clone id2 is still in the scope, so it should not be released yet
            assert!(!IdFactory::instance().get_locked_state().contains(&*id2.0));
            id_copy = id2.into();
        } //drop id1
        let id2 = factory.get_next_id();
        assert_eq!(id2, id_copy);
    }
}
