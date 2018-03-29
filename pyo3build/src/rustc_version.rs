extern crate version_check;
use self::version_check::{is_min_date, is_min_version, supports_features};

// Specifies the minimum nightly version needed to compile pyo3.
const MIN_DATE: &'static str = "2017-11-07";
const MIN_VERSION: &'static str = "1.23.0-nightly";

pub fn check_rustc_version() {
    let ok_channel = supports_features();
    let ok_version = is_min_version(MIN_VERSION);
    let ok_date = is_min_date(MIN_DATE);

    let print_version_err = |version: &str, date: &str| {
        eprintln!(
            "Installed version is: {} ({}). Minimum required: {} ({}).",
            version, date, MIN_VERSION, MIN_DATE
        );
    };

    match (ok_channel, ok_version, ok_date) {
        (Some(ok_channel), Some((ok_version, version)), Some((ok_date, date))) => {
            if !ok_channel {
                eprintln!("Error: pyo3 requires a nightly or dev version of Rust.");
                print_version_err(&*version, &*date);
                panic!("Aborting compilation due to incompatible compiler.")
            }

            if !ok_version || !ok_date {
                eprintln!("Error: pyo3 requires a more recent version of rustc.");
                eprintln!("Use `rustup update` or your preferred method to update Rust");
                print_version_err(&*version, &*date);
                panic!("Aborting compilation due to incompatible compiler.")
            }
        }
        _ => {
            println!(
                "cargo:warning={}",
                "pyo3 was unable to check rustc compatibility."
            );
            println!(
                "cargo:warning={}",
                "Build may fail due to incompatible rustc version."
            );
        }
    }
}
