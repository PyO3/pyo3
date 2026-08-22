use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

// Attaching drains the deferred-decref pool behind a process-global mutex, which serializes
// attaches across threads.
#[cfg(Py_GIL_DISABLED)]
mod scaling {
    use codspeed_criterion_compat::{BenchmarkId, Criterion, Throughput};

    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc,
    };
    use std::thread::{sleep, spawn, JoinHandle};
    use std::time::{Duration, Instant};

    use pyo3::prelude::*;

    const THREADS: [usize; 3] = [1, 2, 4];

    // Nested attaches, so that each iteration is a pool drain rather than a `PyGILState_Ensure`.
    fn bench_nested_attach(c: &mut Criterion, name: &str) {
        let mut group = c.benchmark_group(name);

        for threads in THREADS {
            group.throughput(Throughput::Elements(threads as u64));
            group.bench_function(BenchmarkId::from_parameter(threads), |b| {
                let (done, finished) = channel();

                let senders = (0..threads)
                    .map(|_| {
                        let (sender, receiver) = channel();

                        let done = done.clone();

                        spawn(move || {
                            for iters in receiver {
                                Python::attach(|_| {
                                    for _ in 0..iters {
                                        Python::attach(|_| {});
                                    }
                                });

                                done.send(()).unwrap();
                            }
                        });

                        sender
                    })
                    .collect::<Vec<_>>();

                b.iter_custom(|iters| {
                    let start = Instant::now();

                    for sender in &senders {
                        sender.send(iters).unwrap();
                    }

                    for _ in 0..threads {
                        finished.recv().unwrap();
                    }

                    start.elapsed()
                });
            });
        }

        group.finish();
    }

    // A detached thread trickling drops into the pool, as a real workload would.
    fn spawn_dropper(stop: Arc<AtomicBool>) -> JoinHandle<()> {
        spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let objs =
                    Python::attach(|py| (0..1000).map(|_| py.None()).collect::<Vec<Py<PyAny>>>());

                for obj in objs {
                    drop(obj);

                    sleep(Duration::from_micros(50));
                }
            }
        })
    }

    pub fn benchmarks(c: &mut Criterion) {
        // Drop the returned clone while detached so that the reference pool exists but is empty.
        drop(Python::attach(|py| py.None()));

        bench_nested_attach(c, "nested_attach_scaling/empty_pool");

        let stop = Arc::new(AtomicBool::new(false));
        let dropper = spawn_dropper(stop.clone());

        bench_nested_attach(c, "nested_attach_scaling/sparse_pool");

        stop.store(true, Ordering::Relaxed);
        dropper.join().unwrap();
    }
}

fn criterion_benchmark(_c: &mut Criterion) {
    #[cfg(Py_GIL_DISABLED)]
    #[cfg(not(codspeed))]
    scaling::benchmarks(_c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
