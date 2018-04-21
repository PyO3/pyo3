#![allow(dead_code, unused_variables)]
#![feature(proc_macro, specialization, const_fn, const_align_of, const_size_of)]

extern crate docmatic;

#[test]
fn test_guide() {
    let mut guide_path = ::std::path::PathBuf::new();
    guide_path.push("guide");
    guide_path.push("src");

    for entry in guide_path.read_dir().unwrap() {
        docmatic::assert_file(entry.unwrap().path());
    }
}
