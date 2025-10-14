// this is a clear version of proejct after alot of trail and error methods yes took help of AI for debugging and make it clear also if ur searching for real messy versions of the same project can find in my rust repo.. (which is private as of now but soon to be enclosed)

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use std::thread::sleep;

// ---------------------------------------------------------
//  definining a trait for generic cache behavior
// ---------------------------------------------------------
trait Cache<K, V> {
    fn insert(&mut self, key: K, value: V);
    fn get(&mut self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K);
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------
//  defining cache entry with optional TTL (time to live)
// ---------------------------------------------------------
#[derive(Debug)]
struct CacheEntry<V> {
    value: V,
    expires_at: Option<Instant>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|t| Instant::now() + t);
        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(time) => Instant::now() > time,
            None => false,
        }
    }
}

// ---------------------------------------------------------
// building the memory cache here
// ---------------------------------------------------------
#[derive(Debug)]
struct InMemoryCache<K, V> {
    store: HashMap<K, CacheEntry<V>>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash + Clone, V> InMemoryCache<K, V> {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    fn insert_with_ttl(&mut self, key: K, value: V, ttl: Option<Duration>) {
        self.store.insert(key, CacheEntry::new(value, ttl));
    }

    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<K> = self
            .store
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            self.store.remove(&key);
        }
    }

    fn stats(&self) -> (u64, u64, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }
}

// ---------------------------------------------------------
// implementing cache trait
// ---------------------------------------------------------
impl<K: Eq + Hash + Clone, V> Cache<K, V> for InMemoryCache<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.insert_with_ttl(key, value, None);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        self.cleanup_expired();

        match self.store.get(key) {
            Some(entry) if !entry.is_expired() => {
                self.hits += 1;
                Some(&entry.value)
            }
            _ => {
                self.misses += 1;
                None
            }
        }
    }

    fn remove(&mut self, key: &K) {
        self.store.remove(key);
    }

    fn clear(&mut self) {
        self.store.clear();
    }

    fn len(&self) -> usize {
        self.store.len()
    }
}

// ---------------------------------------------------------
// Add iterator support (for-of style iteration)
// ---------------------------------------------------------
impl<K: Eq + Hash + Clone, V> InMemoryCache<K, V> {
    fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.store
            .iter()
            .filter(|(_, e)| !e.is_expired())
            .map(|(k, e)| (k, &e.value))
    }
}

// ---------------------------------------------------------
//  Computed cache (lazy evaluation) 
// ---------------------------------------------------------
impl<K: Eq + Hash + Clone, V> InMemoryCache<K, V> {
    fn get_or_compute<F>(&mut self, key: K, compute_fn: F) -> &V
    where
        F: FnOnce() -> V,
    {
        if !self.store.contains_key(&key) {
            let val = compute_fn();
            self.insert(key.clone(), val);
        }
        self.get(&key).unwrap()
    }
}

// ---------------------------------------------------------
//  Example of LRU (optional, manual simulation)
// ---------------------------------------------------------
struct SimpleLruCache<K, V> {
    store: HashMap<K, V>,
    order: Vec<K>,
    capacity: usize,
}

impl<K: Eq + Hash + Clone, V> SimpleLruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            store: HashMap::new(),
            order: Vec::new(),
            capacity,
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if self.store.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.store.len() == self.capacity {
            if let Some(oldest) = self.order.first().cloned() {
                self.store.remove(&oldest);
                self.order.remove(0);
            }
        }
        self.order.push(key.clone());
        self.store.insert(key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.store.contains_key(key) {
            self.order.retain(|k| k != key);
            self.order.push(key.clone());
        }
        self.store.get(key)
    }
}

// ---------------------------------------------------------
// final main funcn execution
// ---------------------------------------------------------
fn main() {
    println!("🚀 Generic Cache System Demo\n");

    // ---------- Normal cache ----------
    let mut cache = InMemoryCache::<String, String>::new();

    cache.insert("username".into(), "chandu".into());
    cache.insert_with_ttl("otp".into(), "1234".into(), Some(Duration::from_secs(2)));

    println!("Initial Cache: {:?}", cache.len());

    println!("Fetch username: {:?}", cache.get(&"username".into()));

    println!("Sleeping 3s to expire OTP...");
    sleep(Duration::from_secs(3));
    println!("Fetch otp after expiry: {:?}", cache.get(&"otp".into()));

    let (hits, misses, hit_rate) = cache.stats();
    println!("Stats => Hits: {hits}, Misses: {misses}, HitRate: {hit_rate:.2}%\n");

    // ---------- Computed cache ----------
    let val = cache.get_or_compute("greeting".into(), || "Hello from cache!".into());
    println!("Computed value: {val}");

    println!("\nIterating valid cache entries:");
    for (k, v) in cache.iter() {
        println!("  {k} => {v}");
    }

    // ---------- Simple LRU demo ----------
    println!("\n🧠 Simple LRU Cache:");
    let mut lru = SimpleLruCache::<i32, &str>::new(2);
    lru.insert(1, "one");
    lru.insert(2, "two");
    println!("Cache: {:?}", lru.order);
    lru.get(&1);
    lru.insert(3, "three");
    println!("Cache after accessing 1 and inserting 3: {:?}", lru.order);
}
