#[allow(dead_code)]
#[path = "src/impl_.rs"]
mod impl_;

fn main() {
    // Print out error messages using display, to get nicer formatting.
    if let Err(e) = impl_::configure() {
        eprintln!("error: {}", e);
        std::process::exit(1)
    }
}
