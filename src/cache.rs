use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::Hash;

/*
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
*/

struct CacheValue<V> {
    v: UnsafeCell<V>,
}

pub struct RefCache<K, V> {
    cache: UnsafeCell<HashMap<K, CacheValue<V>>>,
}

impl<K: Eq + Hash, V> RefCache<K, V> {
    pub fn new() -> RefCache<K, V> {
        RefCache {
            cache: UnsafeCell::new(HashMap::new()),
        }
    }

    pub fn put<'a>(&'a self, key: K, value: V) {
        let cache = unsafe { &mut *self.cache.get() };
        let c_value = CacheValue {
            v: UnsafeCell::new(value),
        };
        cache.insert(key, c_value);
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.get_inner(key).map(|cell| unsafe { &*cell.get() })
    }

    pub fn get_mut<'a>(&'a self, key: &K) -> Option<&'a mut V> {
        self.get_inner(key).map(|cell| unsafe { &mut *cell.get() })
    }

    fn get_inner<'a>(&'a self, key: &K) -> Option<&'a UnsafeCell<V>> {
        let cache = unsafe { &*self.cache.get() };
        cache.get(&key).map(|value| &value.v)
    }
}
