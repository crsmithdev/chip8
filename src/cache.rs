use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
struct CacheKey<K> {
    k: *const K,
}

impl<K: Eq> Eq for CacheKey<K> {}

impl<K: Hash> Hash for CacheKey<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for CacheKey<K> {
    fn eq(&self, other: &CacheKey<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

struct CacheValue<K, V> {
    k: K,
    v: V,
}

pub struct RefCache<K, V> {
    cache: UnsafeCell<HashMap<CacheKey<K>, UnsafeCell<V>>>,
}

impl<K: Eq + Hash, V> RefCache<K, V> {
    pub fn new() -> RefCache<K, V> {
        RefCache {
            cache: UnsafeCell::new(HashMap::new()),
        }
    }

    pub fn put<'a>(&'a self, key: K, value: V) {
        let cache = unsafe { &mut *self.cache.get() };
        let k = CacheKey { k: &key };
        cache.insert(k, UnsafeCell::new(value));
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V> {
        let cache = unsafe { &*self.cache.get() };
        let k = CacheKey { k: key };
        cache.get(&k).map(|cell| unsafe { &*cell.get() })
    }

    pub fn get_or_else<'a, F: FnOnce() -> V>(&'a self, key: K, default: F) -> &'a V {
        let cache = unsafe { &*self.cache.get() };
        let k = CacheKey { k: &key };
        let k2 = CacheKey { k: &key };
        match cache.get(&k) {
            Some(cell) => unsafe { &*cell.get() },
            None => {
                let created = default();
                self.put(key, created);
                let cell = cache.get(&k2).unwrap();
                unsafe { &*cell.get() }
            }
        }
    }

    pub fn get_mut<'a>(&'a self, key: K) -> Option<&'a mut V> {
        let cache = unsafe { &*self.cache.get() };
        let k = CacheKey { k: &key };
        cache.get(&k).map(|cell| unsafe { &mut *cell.get() })
    }

    pub fn get_mut_or_else<'a, F: FnOnce() -> V>(&'a self, key: K, default: F) -> &'a mut V {
        let cache = unsafe { &*self.cache.get() };
        let k = CacheKey { k: &key };
        let k2 = CacheKey { k: &key };
        match cache.get(&k) {
            Some(cell) => unsafe { &mut *cell.get() },
            None => {
                let mut created = default();
                self.put(key, created);
                let cell = cache.get(&k2).unwrap();
                unsafe { &mut *cell.get() }
            }
        }
    }
}
