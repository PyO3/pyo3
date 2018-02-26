// Source adopted from
// https://github.com/tildeio/helix-website/blob/master/crates/word_count/src/lib.rs
#![feature(proc_macro, specialization, const_fn)]
#![feature(const_fn, const_align_of, const_size_of, const_ptr_null, const_ptr_null_mut)]

extern crate pyo3;
extern crate rayon;

use std::fs::File;
use std::io::prelude::*;

use rayon::prelude::*;
use pyo3::prelude::*;

#[py::class]
struct WordCounter {
    path: String,
    token: PyToken,
}

#[py::methods]
impl WordCounter {

    #[new]
    fn __new__(obj: &PyRawObject, path: String) -> PyResult<()> {
        obj.init(|t| WordCounter {path: path, token: t})
    }

    fn search(&self, py: Python, search: String) -> PyResult<i32> {
        let mut file = File::open(self.path.as_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let count = py.allow_threads(move || wc_parallel(&contents, &search));
        Ok(count)
    }

    fn search_sequential(&self, search: String) -> PyResult<i32> {
        let mut file = File::open(self.path.as_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(wc_sequential(&contents, &search))
    }
}


fn matches(word: &str, search: &str) -> bool {
    let mut search = search.chars();
    for ch in word.chars().skip_while(|ch| !ch.is_alphabetic()) {
        match search.next() {
            None => { return !ch.is_alphabetic(); }
            Some(expect) => {
                if ch.to_lowercase().next() != Some(expect) {
                    return false;
                }
            }
        }
    }
    return search.next().is_none();
}

fn wc_line(line: &str, search: &str) -> i32 {
    let mut total = 0;
    for word in line.split(' ') {
        if matches(word, search) {
            total += 1;
        }
    }
    total
}

fn wc_sequential(lines: &str, search: &str) -> i32 {
    lines.lines()
         .map(|line| wc_line(line, search))
         .fold(0, |sum, line| sum + line)
}

fn wc_parallel(lines: &str, search: &str) -> i32 {
    lines.par_lines()
         .map(|line| wc_line(line, search))
         .sum()
}



#[py::modinit(_word_count)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WordCounter>()?;

    Ok(())
}
