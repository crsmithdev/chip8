use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::SystemTime;

struct CacheValue<V> {
    v: UnsafeCell<V>,
    // TODO LRU pointers
}

pub struct Cache<K, V> {
    cache: UnsafeCell<HashMap<K, CacheValue<V>>>,
}

impl<K: Eq + Hash, V> Default for Cache<K, V> {
    fn default() -> Self {
        Cache {
            cache: UnsafeCell::new(HashMap::new()),
        }
    }
}

impl<K: Eq + Hash, V> Cache<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&self, key: K, value: V) {
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


pub struct FPSCounter {
    last_frame: SystemTime,
    last_fps: SystemTime,
    fps_actual: f32,
    frames: u32,
    ms_per_frame: f32,
}

impl FPSCounter {
    pub fn new(fps: u32) -> FPSCounter {
        FPSCounter {
            last_frame: SystemTime::now(),
            last_fps: SystemTime::now(),
            fps_actual: 0.0,
            frames: 0,
            ms_per_frame: 1000.0 / fps as f32,
        }
    }

    pub fn frame(&mut self) -> u64 {
        let now = SystemTime::now();
        let delta = now.duration_since(self.last_frame).unwrap().as_millis() as f32;
        self.frames += 1;
        self.last_frame = now;
        (self.ms_per_frame - delta).max(0.0) as u64
    }

    pub fn fps(&mut self) -> f32 {
        let now = SystemTime::now();
        let delta = now.duration_since(self.last_fps).unwrap().as_millis();

        if delta > 1000 {
            self.fps_actual = self.frames as f32 / (delta as f32 / 1000.0);
            self.frames = 0;
            self.last_fps = now;
        }

        self.fps_actual
    }
}
