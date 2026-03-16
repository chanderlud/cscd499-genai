use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG};
use windows::Win32::System::Threading::{
    AcquireSRWLockExclusive, InitializeConditionVariable, InitializeSRWLock,
    ReleaseSRWLockExclusive, SleepConditionVariableSRW, WakeAllConditionVariable,
    WakeConditionVariable, CONDITION_VARIABLE, INFINITE, SRWLOCK,
};

struct BoundedQueue {
    lock: SRWLOCK,
    not_full: CONDITION_VARIABLE,
    not_empty: CONDITION_VARIABLE,
    queue: UnsafeCell<VecDeque<u32>>,
    capacity: u32,
    producers_done: UnsafeCell<bool>,
}

unsafe impl Send for BoundedQueue {}
unsafe impl Sync for BoundedQueue {}

impl BoundedQueue {
    fn new(capacity: u32) -> Self {
        let mut queue = BoundedQueue {
            lock: SRWLOCK::default(),
            not_full: CONDITION_VARIABLE::default(),
            not_empty: CONDITION_VARIABLE::default(),
            queue: UnsafeCell::new(VecDeque::with_capacity(capacity as usize)),
            capacity,
            producers_done: UnsafeCell::new(false),
        };
        unsafe {
            queue.lock = InitializeSRWLock();
            queue.not_full = InitializeConditionVariable();
            queue.not_empty = InitializeConditionVariable();
        }
        queue
    }

    fn push(&self, item: u32) -> Result<()> {
        unsafe {
            AcquireSRWLockExclusive(&self.lock as *const _ as *mut _);

            while (*self.queue.get()).len() as u32 == self.capacity && !*self.producers_done.get() {
                SleepConditionVariableSRW(
                    &self.not_full as *const _ as *mut _,
                    &self.lock as *const _ as *mut _,
                    INFINITE,
                    0,
                )?;
            }

            if *self.producers_done.get() {
                ReleaseSRWLockExclusive(&self.lock as *const _ as *mut _);
                return Ok(());
            }

            (*self.queue.get()).push_back(item);
            WakeConditionVariable(&self.not_empty as *const _ as *mut _);
            ReleaseSRWLockExclusive(&self.lock as *const _ as *mut _);
            Ok(())
        }
    }

    fn pop(&self) -> Result<Option<u32>> {
        unsafe {
            AcquireSRWLockExclusive(&self.lock as *const _ as *mut _);

            while (*self.queue.get()).is_empty() && !*self.producers_done.get() {
                SleepConditionVariableSRW(
                    &self.not_empty as *const _ as *mut _,
                    &self.lock as *const _ as *mut _,
                    INFINITE,
                    0,
                )?;
            }

            if (*self.queue.get()).is_empty() && *self.producers_done.get() {
                ReleaseSRWLockExclusive(&self.lock as *const _ as *mut _);
                return Ok(None);
            }

            let item = (*self.queue.get()).pop_front();
            WakeConditionVariable(&self.not_full as *const _ as *mut _);
            ReleaseSRWLockExclusive(&self.lock as *const _ as *mut _);
            Ok(item)
        }
    }

    fn set_producers_done(&self) {
        unsafe {
            AcquireSRWLockExclusive(&self.lock as *const _ as *mut _);
            *self.producers_done.get() = true;
            WakeAllConditionVariable(&self.not_empty as *const _ as *mut _);
            ReleaseSRWLockExclusive(&self.lock as *const _ as *mut _);
        }
    }
}

pub fn bounded_queue_stress(
    n_items: u32,
    capacity: u32,
    producers: u32,
    consumers: u32,
) -> Result<Vec<u32>> {
    if capacity == 0 || producers == 0 || consumers == 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let queue = Arc::new(BoundedQueue::new(capacity));
    let mut producer_handles = Vec::new();
    let mut consumer_handles = Vec::new();

    // Start producers
    for producer_id in 0..producers {
        let queue = Arc::clone(&queue);
        let items_per_producer = n_items / producers;
        let remainder = if producer_id == producers - 1 {
            n_items % producers
        } else {
            0
        };
        let start = producer_id * items_per_producer;
        let end = start + items_per_producer + remainder;

        producer_handles.push(thread::spawn(move || -> Result<()> {
            for item in start..end {
                queue.push(item)?;
            }
            Ok(())
        }));
    }

    // Start consumers
    for _ in 0..consumers {
        let queue = Arc::clone(&queue);
        consumer_handles.push(thread::spawn(move || -> Result<Vec<u32>> {
            let mut consumed = Vec::new();
            loop {
                match queue.pop()? {
                    Some(item) => consumed.push(item),
                    None => break,
                }
            }
            Ok(consumed)
        }));
    }

    // Wait for all producers to finish
    for handle in producer_handles {
        handle.join().map_err(|_| Error::from_hresult(E_FAIL))??;
    }

    // Signal that producers are done
    queue.set_producers_done();

    // Collect results from consumers
    let mut all_consumed = Vec::new();
    for handle in consumer_handles {
        let consumed = handle.join().map_err(|_| Error::from_hresult(E_FAIL))??;
        all_consumed.extend(consumed);
    }

    Ok(all_consumed)
}