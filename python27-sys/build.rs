extern crate pkg_config;

fn main() {
    pkg_config::find_library("python-2.7").unwrap();
}

