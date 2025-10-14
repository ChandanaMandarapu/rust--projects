// just a project with alot of practice 

// borrowcheckers cache entr cache struct and new + insert 
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
}

pub struct Cache<K, V>
where
    K: Eq + Hash,
{
    store: HashMap<K, CacheEntry<V>>,
    hits: u64,
    misses: u64,
    ttl: Option<Duration>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    pub fn new(ttl: Option<Duration>) -> Self {
        Cache {
            store: HashMap::new(),
            hits: 0,
            misses: 0,
            ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
        };
        self.store.insert(key, entry);
    }

    // if an entry is expired (false if cache TTL is None)
    fn is_expired(&self, entry: &CacheEntry<V>) -> bool {
        if let Some(ttl) = self.ttl {
            entry.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    /// Borrow-safe 
    pub fn get(&mut self, key: &K) -> Option<&V> {
        //  d a short-lived read to check presence/expiry
        let maybe_expired = {
            if let Some(entry) = self.store.get(key) {
                if self.is_expired(entry) {
                    Some(true) // present but expired
                } else {
                    Some(false) // present and valid
                }
            } else {
                None // absent - am sorry not present here
            }
        };

        match maybe_expired {
            None => {
                // Not found
                self.misses += 1;
                None
            }
            Some(true) => {
                // expired -> removed under a fresh mutable borrow
                self.store.remove(key);
                self.misses += 1;
                None
            }
            Some(false) => {
                // present and valid -> count a hit and return a fresh borrow
                self.hits += 1;
                self.store.get(key).map(|entry| &entry.value)
            }
        }
    }
}
