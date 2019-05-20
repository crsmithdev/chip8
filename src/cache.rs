use std::collections::HashMap;
use std::rc::Rc;

pub struct RcCache<T> {
    cache: HashMap<String, Rc<T>>,
}

impl<T> RcCache<T> {
    pub fn new() -> RcCache<T> {
        RcCache {
            cache: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<Rc<T>> {
        self.cache.get(key).map(|x| x.clone())
    }

    fn put(&mut self, key: &str, value: Rc<T>) {
        self.cache.insert(key.to_owned(), value);
    }
}

pub struct RefCache {
    cache: HashMap<String, CacheObj>,
}

impl RefCache {
    pub fn new() -> RefCache {
        RefCache {
            cache: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&CacheObj> {
        self.cache.get(key)
    }

    fn put(&mut self, key: &str, value: CacheObj) {
        self.cache.insert(key.to_owned(), value);
    }
}