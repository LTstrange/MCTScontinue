use std::sync::atomic::{AtomicPtr, Ordering};

use rand::Rng;

pub struct AtomicBox<T>(AtomicPtr<T>);

impl<T> Default for AtomicBox<T> {
    fn default() -> Self {
        Self(AtomicPtr::default())
    }
}

impl<T> AtomicBox<T> {
    // Tries to set the AtomicBox to this value if empty.
    // Returns a reference to whatever is in the box.
    pub(super) fn try_set(&self, value: Box<T>) -> &T {
        let ptr = Box::into_raw(value);
        // Try to replace nullptr with the value.
        let ret_ptr = if let Err(new_ptr) = self.0.compare_exchange(
            std::ptr::null_mut(),
            ptr,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            // If someone beat us to it, return the original drop the new one.
            unsafe { drop(Box::from_raw(ptr)) };
            new_ptr
        } else {
            ptr
        };
        unsafe { ret_ptr.as_ref().unwrap() }
    }

    pub(super) fn get(&self) -> Option<&T> {
        let ptr = self.0.load(Ordering::Relaxed);
        unsafe { ptr.as_ref() }
    }
}

impl<T> Drop for AtomicBox<T> {
    fn drop(&mut self) {
        let ptr = *self.0.get_mut();
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}

static PRIMES: [usize; 16] = [
    14323, 18713, 19463, 30553, 33469, 45343, 50221, 51991, 53201, 56923, 64891, 72763, 74471,
    81647, 92581, 94693,
];

// Find and return the highest scoring element of the set.
// If multiple elements have the highest score, select one randomly.
// Constraints:
//   - Don't call the scoring function more than once per element.
//   - Select one uniformly, so that a run of high scores doesn't
//       bias towards the one that scans first.
//   - Don't shuffle the input or allocate a new array for shuffling.
//   - Optimized for sets with <10k values.
pub(super) fn random_best<T, F: Fn(&T) -> f32>(set: &[T], score_fn: F) -> Option<&T> {
    // To make the choice more uniformly random among the best moves,
    // start at a random offset and stride by a random amount.
    // The stride must be coprime with n, so pick from a set of 5 digit primes.

    let n = set.len();
    // Combine both random numbers into a single rng call.
    let r = rand::thread_rng().gen_range(0..n * PRIMES.len());
    let mut i = r / PRIMES.len();
    let stride = PRIMES[r % PRIMES.len()];

    let mut best_score = f32::NEG_INFINITY;
    let mut best = None;
    for _ in 0..n {
        let score = score_fn(&set[i]);
        debug_assert!(!score.is_nan());
        if score > best_score {
            best_score = score;
            best = Some(&set[i]);
        }
        i = (i + stride) % n;
    }
    best
}
