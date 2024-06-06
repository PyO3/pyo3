use codspeed_criterion_compat::{criterion_group, criterion_main, BatchSize, Bencher, Criterion};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::channel,
    Arc, Barrier,
};
use std::thread::spawn;
use std::time::{Duration, Instant};

use pyo3::prelude::*;

fn drop_many_objects(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| {
            for _ in 0..1000 {
                drop(py.None());
            }
        });
    });
}

fn drop_many_objects_without_gil(b: &mut Bencher<'_>) {
    b.iter_batched(
        || {
            Python::with_gil(|py| {
                (0..1000)
                    .map(|_| py.None().into_py(py))
                    .collect::<Vec<PyObject>>()
            })
        },
        |objs| {
            drop(objs);

            Python::with_gil(|_py| ());
        },
        BatchSize::SmallInput,
    );
}

fn drop_many_objects_multiple_threads(b: &mut Bencher<'_>) {
    const THREADS: usize = 5;

    let barrier = Arc::new(Barrier::new(1 + THREADS));

    let done = Arc::new(AtomicUsize::new(0));

    let sender = (0..THREADS)
        .map(|_| {
            let (sender, receiver) = channel();

            let barrier = barrier.clone();

            let done = done.clone();

            spawn(move || {
                for objs in receiver {
                    barrier.wait();

                    drop(objs);

                    done.fetch_add(1, Ordering::AcqRel);
                }
            });

            sender
        })
        .collect::<Vec<_>>();

    b.iter_custom(|iters| {
        let mut duration = Duration::ZERO;

        let mut last_done = done.load(Ordering::Acquire);

        for _ in 0..iters {
            for sender in &sender {
                let objs = Python::with_gil(|py| {
                    (0..1000 / THREADS)
                        .map(|_| py.None().into_py(py))
                        .collect::<Vec<PyObject>>()
                });

                sender.send(objs).unwrap();
            }

            barrier.wait();

            let start = Instant::now();

            loop {
                Python::with_gil(|_py| ());

                let done = done.load(Ordering::Acquire);
                if done - last_done == THREADS {
                    last_done = done;
                    break;
                }
            }

            Python::with_gil(|_py| ());

            duration += start.elapsed();
        }

        duration
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("drop_many_objects", drop_many_objects);
    c.bench_function(
        "drop_many_objects_without_gil",
        drop_many_objects_without_gil,
    );
    c.bench_function(
        "drop_many_objects_multiple_threads",
        drop_many_objects_multiple_threads,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
