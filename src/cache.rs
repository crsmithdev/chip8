use sdl2::render::Texture;
use std::cell::UnsafeCell;
use std::collections::HashMap;

pub type TextureCache = RefCache<Texture>;

pub struct RefCache<T> {
    cache: UnsafeCell<HashMap<String, UnsafeCell<T>>>,
}

impl<T> RefCache<T> {
    pub fn new() -> RefCache<T> {
        RefCache {
            cache: UnsafeCell::new(HashMap::new()),
        }
    }

    pub fn put<'a>(&'a self, key: &'_ str, value: T) {
        let cache = unsafe { &mut *self.cache.get() };
        cache.insert(key.to_owned(), UnsafeCell::new(value));
    }

    pub fn get<'a>(&'a self, key: &'_ str) -> Option<&'a T> {
        let cache = unsafe { &*self.cache.get() };
        cache.get(key).map(|cell| unsafe { &*cell.get() })
    }

    pub fn get_or_else<'a, F: FnOnce() -> T>(&'a self, key: &'_ str, default: F) -> Option<&'a T> {
        let cache = unsafe { &*self.cache.get() };
        match cache.get(key) {
            Some(cell) => Some(unsafe { &*cell.get() }),
            None => {
                self.put(key, default());
                self.get(key)
            }
        }
    }

    pub fn get_mut<'a>(&'a self, key: &'_ str) -> Option<&'a mut T> {
        let cache = unsafe { &*self.cache.get() };
        cache.get(key).map(|cell| unsafe { &mut *cell.get() })
    }

    pub fn get_mut_or_else<'a, F: FnOnce() -> T>(
        &'a self,
        key: &'_ str,
        default: F,
    ) -> Option<&'a mut T> {
        let cache = unsafe { &*self.cache.get() };
        match cache.get(key) {
            Some(cell) => Some(unsafe { &mut *cell.get() }),
            None => {
                let mut value = default();
                self.put(key, value);
                self.get_mut(key)
            }
        }
    }
}
