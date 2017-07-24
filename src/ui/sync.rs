use render;

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
    mutex: Arc<Mutex<std::vec::Vec<render::FilmPixel>>>,
}

impl SharedData {
    pub fn new(film: &render::Film) -> SharedData {
        SharedData {
            width: film.width,
            height: film.height,
            has_new: Arc::new(AtomicBool::new(false)),
            mutex: Arc::new(Mutex::new(vec![render::FilmPixel::zero(); film.width * film.height]))
        }
    }

    pub fn store(&self, film: &render::Film) {
        if !self.has_new.load(atomic::Ordering::Relaxed) {
            let mut data = self.mutex.lock().unwrap();
            data.copy_from_slice(&film.pixels);
            self.has_new.store(true, atomic::Ordering::Relaxed);
        }
    }

    pub fn load<'a>(&'a self) -> Option<SharedDataGuard> {
        if self.has_new.load(atomic::Ordering::Relaxed) {
            Some(SharedDataGuard {
                shared_data: self
            })
        }
        else {
            None
        }
    }
}

pub struct SharedDataGuard<'a> {
    shared_data: &'a SharedData
}

impl<'a> SharedDataGuard<'a> {
    pub fn get(&'a self) -> std::sync::MutexGuard<std::vec::Vec<render::FilmPixel>> {
        self.shared_data.mutex.lock().unwrap()
    }
}

impl<'a> Drop for SharedDataGuard<'a> {
    fn drop(&mut self) {
        self.shared_data.has_new.store(false, atomic::Ordering::Relaxed);
    }
}
