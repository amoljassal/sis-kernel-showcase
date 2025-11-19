#![cfg(feature = "loom-tests")]

// Loom-based concurrency tests for percpu-like patterns

#[cfg(test)]
mod tests {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    // Simple percpu-like storage using atomics to simulate per-hart counters
    struct SimPerCpu {
        slots: [AtomicUsize; 4],
    }

    impl SimPerCpu {
        fn new() -> Self {
            Self { slots: [AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)] }
        }
        fn get_local(&self, cpu: usize) -> &AtomicUsize { &self.slots[cpu % 4] }
        fn for_each<F: FnMut(usize, &AtomicUsize)>(&self, mut f: F) {
            for i in 0..4 { f(i, &self.slots[i]); }
        }
    }

    #[test]
    fn percpu_increment_no_race() {
        loom::model(|| {
            let percpu = Arc::new(SimPerCpu::new());
            let mut handles = vec![];

            for cpu in 0..4 {
                let p = percpu.clone();
                handles.push(thread::spawn(move || {
                    // Each CPU increments its own slot 10 times
                    for _ in 0..10 {
                        p.get_local(cpu).fetch_add(1, Ordering::SeqCst);
                        thread::yield_now();
                    }
                }));
            }
            for h in handles { h.join().unwrap(); }

            // Sum should be 40 regardless of interleaving
            let mut sum = 0;
            percpu.for_each(|_, slot| sum += slot.load(Ordering::SeqCst));
            assert_eq!(sum, 40);
        });
    }
}

