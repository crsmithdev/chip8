use sdl2::render::Texture;
use std::cell::RefCell;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type TextureCache = RcCache<Texture>;

pub struct RcCache<T> {
    immutable: UnsafeCell<HashMap<String, Rc<T>>>,
    mutable: HashMap<String, Rc<RefCell<T>>>,
}

impl<T> RcCache<T> {
    pub fn new() -> RcCache<T> {
        RcCache {
            immutable: UnsafeCell::new(HashMap::new()),
            mutable: HashMap::new(),
        }
    }

    pub fn put(&self, key: &str, value: T) {
        let mut cache = unsafe { &mut *self.immutable.get() };
        cache.insert(key.to_owned(), Rc::new(value));
    }

    pub fn get(&self, key: &str) -> Option<Rc<T>> {
        let cache = unsafe { &*self.immutable.get() };
        cache.get(key).map(|x| x.clone())
    }
    /*
    pub fn put_mut(&mut self, key: &str, value: T) {
        self.mutable
            .insert(key.to_owned(), Rc::new(RefCell::new(value)));
    }

    pub fn get_mut(&self, key: &str) -> Option<Rc<RefCell<T>>> {
        self.mutable.get(key).map(|x| x.clone())
    }
    */
}
