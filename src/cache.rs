use std::collections::HashMap;
use std::cell::UnsafeCell;
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

    pub fn get(&self, key: &str) -> Option<Rc<T>> {
        self.cache.get(key).map(|x| x.clone())
    }

    pub fn put(&mut self, key: &str, value: Rc<T>) {
        self.cache.insert(key.to_owned(), value);
    }
}

pub struct RefCache<T> {
    cache: HashMap<String, UnsafeCell<T>>,
}

impl<T> RefCache<T> {
    pub fn new() -> RefCache<T> {
        RefCache {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        let cell = self.cache.get(key);
        match cell {
            Some(c) => {
                let val: &T = unsafe { &*c.get()};
                Some(val)
            },
            None => None,
        }
    }

    pub fn get_mut(&self, key: &str) -> Option<&mut T> {
        let cell = self.cache.get(key);
        match cell {
            Some(c) => {
                let val: &mut T = unsafe { &mut *c.get()};
                Some(val)
            },
            None => None,
        }
    }

    pub fn put(&mut self, key: &str, value: T) {
        self.cache.insert(key.to_owned(), UnsafeCell::new(value));
    }
}