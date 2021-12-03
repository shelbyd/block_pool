//! A simple object pool that blocks when taking an item out.
//!
//! ```
//! use block_pool::Pool;
//!
//! let pool = Pool::new(vec![1, 2, 3]);
//! let mut item = pool.take();
//! *item += 1;
//! drop(item);
//! ```

use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    sync::{Condvar, Mutex},
};

/// Container for objects that can be taken out.
pub struct Pool<T> {
    items: Mutex<VecDeque<T>>,
    value_returned: Condvar,
}

impl<T> Pool<T> {
    /// Construct a new Pool with the items from the iterator.
    pub fn new(items: impl IntoIterator<Item = T>) -> Self {
        Pool {
            items: Mutex::new(items.into_iter().collect()),
            value_returned: Condvar::new(),
        }
    }

    /// Remove an item from the pool, this will take the "oldest" item.
    ///
    /// The item will automatically get returned to the pool when the smart pointer is dropped.
    ///
    /// There is no "resetting" that is common in other frameworks. You need to perform any
    /// resetting on your own.
    pub fn take(&self) -> Returnable<T> {
        let mut lock = self.items.lock().unwrap();
        loop {
            if let Some(value) = lock.pop_front() {
                return Returnable {
                    value: Some(value),
                    pool: self,
                };
            }
            lock = self.value_returned.wait(lock).unwrap();
        }
    }

    fn return_(&self, value: T) {
        self.items.lock().unwrap().push_back(value);
    }
}

/// A smart pointer that holds an object taken from a pool.
///
/// Returns the object to the pool when dropped.
pub struct Returnable<'p, T> {
    // Only Option so that we can take ownership of the value in Drop.
    value: Option<T>,
    pool: &'p Pool<T>,
}

impl<'p, T> Drop for Returnable<'p, T> {
    fn drop(&mut self) {
        self.pool.return_(self.value.take().unwrap());
        self.pool.value_returned.notify_one();
    }
}

impl<'p, T> Deref for Returnable<'p, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<'p, T> DerefMut for Returnable<'p, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}
