use pyo3::prelude::*;
use rayon::prelude::*;

/// Searches for the word, parallelized by rayon
#[pyfunction]
fn search(contents: &str, needle: &str) -> usize {
    contents
        .par_lines()
        .map(|line| count_line(line, needle))
        .sum()
}

/// Searches for a word in a classic sequential fashion
#[pyfunction]
fn search_sequential(contents: &str, needle: &str) -> usize {
    contents.lines().map(|line| count_line(line, needle)).sum()
}

#[pyfunction]
fn search_sequential_allow_threads(py: Python<'_>, contents: &str, needle: &str) -> usize {
    py.allow_threads().with(|| search_sequential(contents, needle))
}

/// Count the occurrences of needle in line, case insensitive
fn count_line(line: &str, needle: &str) -> usize {
    let mut total = 0;
    for word in line.split(' ') {
        if word == needle {
            total += 1;
        }
    }
    total
}

#[pymodule]
fn word_count(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(search_sequential, m)?)?;
    m.add_function(wrap_pyfunction!(search_sequential_allow_threads, m)?)?;

    Ok(())
}
