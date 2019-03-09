use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display};
use std::path::Path;
use utils::{parse_header_defines, run_python_script};

#[derive(Debug, Clone)]
pub enum PythonInterpreterKind {
    CPython,
    PyPy,
}

#[derive(Debug, Clone)]
pub struct PythonVersion {
    pub major: u8,
    // minor == None means any minor version will do
    pub minor: Option<u8>,
    // kind = "pypy" or "cpython" are supported, default to cpython
    pub kind: PythonInterpreterKind,
}

impl PartialEq for PythonVersion {
    fn eq(&self, o: &PythonVersion) -> bool {
        self.major == o.major && (self.minor.is_none() || self.minor == o.minor)
    }
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.major.fmt(f));
        try!(f.write_str("."));
        match self.minor {
            Some(minor) => try!(minor.fmt(f)),
            None => try!(f.write_str("*")),
        };
        Ok(())
    }
}

impl Default for PythonVersion {
    fn default() -> Self {
        PythonVersion {
            major: 3,
            minor: None,
            kind: PythonInterpreterKind::CPython,
        }
    }
}

impl PythonVersion {
    /// Determine the python version we're supposed to be building
    /// from the features passed via the environment.
    ///
    /// The environment variable can choose to omit a minor
    /// version if the user doesn't care.
    pub fn from_env() -> Result<Self, String> {
        let re = Regex::new(r"CARGO_FEATURE_PYTHON(\d+)(_(\d+))?").expect("This is a valid regex");

        let interpreter_kind = if cfg!(feature = "pypy") {
            PythonInterpreterKind::PyPy
        } else {
            PythonInterpreterKind::CPython
        };

        // sort env::vars so we get more explicit version specifiers first
        // so if the user passes e.g. the python-3 feature and the python-3-5
        // feature, python-3-5 takes priority.
        let mut vars = env::vars().collect::<Vec<_>>();

        vars.sort_by(|a, b| b.cmp(&a));
        for (key, _) in vars {
            match re.captures(&key) {
                Some(cap) => {
                    return Ok(PythonVersion {
                        kind: interpreter_kind,
                        major: cap.get(1).expect("must match").as_str().parse().unwrap(),
                        minor: match cap.get(3) {
                            Some(s) => Some(s.as_str().parse().unwrap()),
                            None => None,
                        },
                    });
                }
                None => (),
            }
        }

        Err(
            "Python version feature was not found. At least one python version \
             feature must be enabled."
                .to_owned(),
        )
    }

    pub fn from_cross_env(header_defines: &HashMap<String, String>) -> Result<Self, String> {
        let major = header_defines
            .get("PY_MAJOR_VERSION")
            .and_then(|major| major.parse::<u8>().ok())
            .ok_or("PY_MAJOR_VERSION undefined".to_string())?;

        let minor = header_defines
            .get("PY_MINOR_VERSION")
            .and_then(|minor| minor.parse::<u8>().ok())
            .ok_or("PY_MINOR_VERSION undefined".to_string())?;

        Ok(PythonVersion {
            major,
            minor: Some(minor),
            kind: PythonInterpreterKind::CPython,
        })
    }

    /// Returns a name of possible python binary names.
    /// Ex. vec![python, python3, python3.5]
    pub fn possible_binary_names(&self) -> Vec<String> {
        let mut possible_names = vec![];

        let binary_name = format!("{:?}", self.kind).to_ascii_lowercase();

        possible_names.push(binary_name.clone());
        possible_names.push(format!("{}{}", &binary_name, self.major));

        if let Some(minor) = self.minor {
            possible_names.push(format!("{}{}.{}", &binary_name, self.major, minor));
        }

        possible_names
    }

    pub fn from_interpreter(interpreter_path: impl AsRef<Path>) -> Result<Self, String> {
        let script = "import sys;\
                      print('__pypy__' in sys.builtin_module_names);\
                      print(sys.version_info[0:2]);";

        let out = run_python_script(interpreter_path.as_ref(), script)?;
        let lines: Vec<&str> = out.lines().collect();

        let is_pypy: bool = lines[0]
            .to_ascii_lowercase()
            .parse()
            .expect("Should print a bool");
        let (major, minor) = PythonVersion::parse_interpreter_version(lines[1])?;

        Ok(Self {
            major,
            minor: Some(minor),
            kind: if is_pypy {
                PythonInterpreterKind::PyPy
            } else {
                PythonInterpreterKind::CPython
            },
        })
    }

    /// Parse string as interpreter version.
    fn parse_interpreter_version(line: &str) -> Result<(u8, u8), String> {
        let version_re = Regex::new(r"\((\d+), (\d+)\)").unwrap();
        match version_re.captures(&line) {
            Some(cap) => Ok((
                cap.get(1).unwrap().as_str().parse().unwrap(),
                cap.get(2).unwrap().as_str().parse().unwrap(),
            )),
            None => Err(format!("Unexpected response to version query {}", line)),
        }
    }
}
