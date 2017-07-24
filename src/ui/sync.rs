use std;
use std::ops::Drop;
use std::sync::{Arc, Mutex};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub struct SharedData {
    pub width: usize,
    pub height: usize,
    has_new: Arc<AtomicBool>,
    mutex: Arc<Mutex<std::vec::Vec<[u8; 4]>>>,
}

impl SharedData {
    pub fn new(width: usize, height: usize) -> SharedData {
        SharedData {
            width: width,
            height: height,
            has_new: Arc::new(AtomicBool::new(false)),
            mutex: Arc::new(Mutex::new(vec![[0u8; 4]; width * height]))
        }
    }

    pub fn store(&self) -> Option<SharedDataGuard> {
        // if !self.has_new.load(atomic::Ordering::Relaxed) {
        //     let mut data = self.mutex.lock().unwrap();
        //     data.copy_from_slice(&film.pixels);
        //     self.has_new.store(true, atomic::Ordering::Relaxed);
        // }
        if !self.has_new.load(atomic::Ordering::Relaxed) {
            Some(SharedDataGuard {
                shared_data: self,
                has_new_on_drop: true
            })
        }
        else {
            None
        }
    }

    pub fn load<'a>(&'a self) -> Option<SharedDataGuard> {
        if self.has_new.load(atomic::Ordering::Relaxed) {
            Some(SharedDataGuard {
                shared_data: self,
                has_new_on_drop: false
            })
        }
        else {
            None
        }
    }
}

pub struct SharedDataGuard<'a> {
    shared_data: &'a SharedData,
    has_new_on_drop: bool,
}

impl<'a> SharedDataGuard<'a> {
    pub fn get(&'a self) -> std::sync::MutexGuard<std::vec::Vec<[u8; 4]>> {
        self.shared_data.mutex.lock().unwrap()
    }
}

impl<'a> Drop for SharedDataGuard<'a> {
    fn drop(&mut self) {
        self.shared_data.has_new.store(self.has_new_on_drop, atomic::Ordering::Relaxed);
    }
}
