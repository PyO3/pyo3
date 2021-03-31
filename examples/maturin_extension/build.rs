use std::process::Command;

fn main() {
    let out = Command::new("python")
        .args(&["-c", "import sys; import platform; print(sys.version_info[1]); print(platform.python_implementation())"])
        .output()
        .expect("python version did not print");

    let output = String::from_utf8_lossy(&out.stdout);
    let mut lines = output.trim().lines();

    println!("{}", output);

    let version: u8 = lines
        .next()
        .unwrap()
        .parse()
        .expect("python version was not parsed");
    let implementation = lines.next().unwrap();

    for each in 6..version {
        println!("cargo:rustc-cfg=Py_3_{}", each);
    }

    if implementation == "PyPy" {
        println!("cargo:rustc-cfg=PyPy");
    }
}
