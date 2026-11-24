use std::hint::black_box;

use codspeed_criterion_compat::{Bencher, Criterion, criterion_group, criterion_main};

use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyTime, PyTimeAccess};

fn date_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let date = PyDate::new(py, 2020, 6, 15).unwrap();
        bench.iter(|| {
            let d = black_box(&date);
            (d.get_year(), d.get_month(), d.get_day())
        });
    });
}

fn datetime_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let datetime = PyDateTime::new(py, 2020, 6, 15, 12, 30, 45, 123_456, None).unwrap();
        bench.iter(|| {
            let dt = black_box(&datetime);
            (
                dt.get_year(),
                dt.get_month(),
                dt.get_day(),
                dt.get_hour(),
                dt.get_minute(),
                dt.get_second(),
                dt.get_microsecond(),
            )
        });
    });
}

fn time_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let time = PyTime::new(py, 12, 30, 45, 123_456, None).unwrap();
        bench.iter(|| {
            let t = black_box(&time);
            (
                t.get_hour(),
                t.get_minute(),
                t.get_second(),
                t.get_microsecond(),
            )
        });
    });
}

fn delta_fields(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let delta = PyDelta::new(py, 1, 3_600, 123_456, false).unwrap();
        bench.iter(|| {
            let d = black_box(&delta);
            (d.get_days(), d.get_seconds(), d.get_microseconds())
        });
    });
}

fn extract_chrono_date(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let date = PyDate::new(py, 2020, 6, 15).unwrap().into_any();
        bench.iter(|| black_box(&date).extract::<chrono::NaiveDate>().unwrap());
    });
}

fn extract_chrono_datetime(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let datetime = PyDateTime::new(py, 2020, 6, 15, 12, 30, 45, 123_456, None)
            .unwrap()
            .into_any();
        bench.iter(|| {
            black_box(&datetime)
                .extract::<chrono::NaiveDateTime>()
                .unwrap()
        });
    });
}

fn extract_chrono_time(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let time = PyTime::new(py, 12, 30, 45, 123_456, None)
            .unwrap()
            .into_any();
        bench.iter(|| black_box(&time).extract::<chrono::NaiveTime>().unwrap());
    });
}

fn extract_chrono_timedelta(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let delta = PyDelta::new(py, 1, 3_600, 123_456, false)
            .unwrap()
            .into_any();
        bench.iter(|| black_box(&delta).extract::<chrono::TimeDelta>().unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("date_fields", date_fields);
    c.bench_function("datetime_fields", datetime_fields);
    c.bench_function("time_fields", time_fields);
    c.bench_function("delta_fields", delta_fields);
    c.bench_function("extract_chrono_date", extract_chrono_date);
    c.bench_function("extract_chrono_datetime", extract_chrono_datetime);
    c.bench_function("extract_chrono_time", extract_chrono_time);
    c.bench_function("extract_chrono_timedelta", extract_chrono_timedelta);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
