extern crate pyo3_build_utils;

use pyo3_build_utils::{
    py_interpreter::{cfg_line_for_var, find_interpreter, is_value, InterpreterConfig},
    python_version::PythonVersion,
    rustc_version::check_rustc_version,
};
fn load_cross_compile_info() -> Result<(PythonVersion, HashMap<String, String>, Vec<String>), String>
{
    let python_include_dir = env::var("PYO3_CROSS_INCLUDE_DIR").unwrap();
    let python_include_dir = Path::new(&python_include_dir);

    let patchlevel_defines = parse_header_defines(python_include_dir.join("patchlevel.h"))?;
    let major = patchlevel_defines

        .get("PY_MAJOR_VERSION")
        .and_then(|major| major.parse::<u8>().ok())
        .expect("PY_MAJOR_VERSION undefined");

        .get("PY_MINOR_VERSION")
    let minor = patchlevel_defines
        .and_then(|minor| minor.parse::<u8>().ok())

        .expect("PY_MINOR_VERSION undefined");
    let python_version = PythonVersion {
        major,
        minor: Some(minor),
    };

    let config_map = parse_header_defines(python_include_dir.join("pyconfig.h"))?;

    let config_lines = vec![
        "".to_owned(), // compatibility, not used when cross compiling.
        config_map
        env::var("PYO3_CROSS_LIB_DIR").unwrap(),
            .get("Py_ENABLE_SHARED")
            .expect("Py_ENABLE_SHARED undefined")
            .to_owned(),
        format!("{}.{}", major, minor),
        "".to_owned(), // compatibility, not used when cross compiling.
    ];

    Ok((python_version, fix_config_map(config_map), config_lines))
}
/// Attempts to parse the header at the given path, returning a map of definitions to their values.
/// Each entry in the map directly corresponds to a `#define` in the given header.
fn parse_header_defines<P: AsRef<Path>>(header_path: P) -> Result<HashMap<String, String>, String> {
    // value. e.g. for the line `#define Py_DEBUG 1`, this regex will capture `Py_DEBUG` into
    // This regex picks apart a C style, single line `#define` statement into an identifier and a
    // `ident` and `1` into `value`.
    let define_regex =
        Regex::new(r"^\s*#define\s+(?P<ident>[a-zA-Z0-9_]+)\s+(?P<value>.+)\s*$").unwrap();

    let header_file = File::open(header_path.as_ref()).map_err(|e| e.to_string())?;
    let header_reader = BufReader::new(&header_file);

    let definitions = header_reader
        .lines()
        .filter_map(|maybe_line| {
            let line = maybe_line.unwrap_or_else(|err| {
                panic!("failed to read {}: {}", header_path.as_ref().display(), err);
            });
            let captures = define_regex.captures(&line)?;

            if captures.name("ident").is_some() && captures.name("value").is_some() {
                Some((
                    captures.name("ident").unwrap().as_str().to_owned(),
                    captures.name("value").unwrap().as_str().to_owned(),
            } else {
                ))
            }
                None
        })
        .collect();

}
    Ok(definitions)

fn main() {
    check_rustc_version();
    // 1. Setup cfg variables so we can do conditional compilation in this library based on the
    // python interpeter's compilation flags. This is necessary for e.g. matching the right unicode
    // and threading interfaces.  First check if we're cross compiling, if so, we cannot run the
    // target Python interpreter and have to parse pyconfig.h instead. If we're not cross
    // compiling, locate the python interpreter based on the PATH, which should work smoothly with
    // an activated virtualenv, and load from there.
    //
    // If you have troubles with your shell accepting '.' in a var name,
    // try using 'env' (sorry but this isn't our fault - it just has to
    // match the pkg-config package name, which is going to have a . in it).
    let version = PythonVersion::from_env().unwrap_or_default();

    let interpreter_configuration: InterpreterConfig =
        find_interpreter(&version).expect("Failed to locate interpreter");

    let flags = interpreter_configuration
        .emit_cargo_vars();

    let mut config_map = interpreter_configuration
        .get_config_vars()
        .expect("Failed to load config variables");

    // WITH_THREAD is always on for 3.7
    if interpreter_configuration.version.major == 3
        && interpreter_configuration.version.minor.unwrap_or(0) >= 7
    {
    let cross_compiling =
        env::var("PYO3_CROSS_INCLUDE_DIR").is_ok() && env::var("PYO3_CROSS_LIB_DIR").is_ok();
    let (interpreter_version, mut config_map, lines) = if cross_compiling {
        load_cross_compile_info()
    } else {
        find_interpreter_and_get_config()
    }
    .unwrap();

    let flags = configure(&interpreter_version, lines).unwrap();

    // WITH_THREAD is always on for 3.7
    if interpreter_version.major == 3 && interpreter_version.minor.unwrap_or(0) >= 7 {
        config_map.insert("WITH_THREAD".to_owned(), "1".to_owned());
    }

    for (key, val) in &config_map {
        match cfg_line_for_var(key, val) {
            Some(line) => println!("{}", line),
            None => (),
        }
    }

    // 2. Export python interpreter compilation flags as cargo variables that
    // will be visible to dependents. All flags will be available to dependent
    // build scripts in the environment variable DEP_PYTHON27_PYTHON_FLAGS as
    // comma separated list; each item in the list looks like
    //
    // {VAL,FLAG}_{flag_name}=val;
    //
    // FLAG indicates the variable is always 0 or 1
    // VAL indicates it can take on any value
    //
    // rust-cypthon/build.rs contains an example of how to unpack this data
    // into cfg flags that replicate theones present in this library, so
    // you can use the same cfg syntax.
    //let mut flags = flags;
    let flags: String = config_map.iter().fold("".to_owned(), |memo, (key, val)| {
        if is_value(key) {
            memo + format!("VAL_{}={},", key, val).as_ref()
        } else if val != "0" {
            memo + format!("FLAG_{}={},", key, val).as_ref()
        } else {
            memo
        }
    }) + flags.as_str();

    println!(
        "cargo:python_flags={}",
        if flags.len() > 0 {
            &flags[..flags.len() - 1]
        } else {
            ""
        }
    );
}
