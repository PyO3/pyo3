use std::{
    collections::HashSet,
    env, fs,
    path::{Component, Path, PathBuf},
};

const FORBIDDEN: &[&str] = &[
    "PyRustPython",
    "#[cfg(PyRustPython)]",
    "#[cfg(not(PyRustPython))]",
    "rustpython_storage",
    "PyRustPython_",
    "ObjExt",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives under third_party/pyo3-fork/xtask")
        .to_path_buf()
}

fn load_allowlist(root: &Path) -> HashSet<String> {
    fs::read_to_string(root.join("tools/backend-boundary-allowlist.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn is_frontend_path(rel: &Path) -> bool {
    rel.extension().is_some_and(|ext| ext == "rs")
        && rel.file_name().and_then(|name| name.to_str()) != Some("tests.rs")
        && !rel.components().any(|c| c == Component::Normal("backend".as_ref()))
        && !rel.components().any(|c| c == Component::Normal("tests".as_ref()))
}

fn main() -> anyhow::Result<()> {
    let cmd = env::args().nth(1).unwrap_or_default();
    anyhow::ensure!(cmd == "check-backend-boundary", "expected `check-backend-boundary`");

    let root = repo_root();
    let allowlist = load_allowlist(&root);
    let mut violations = Vec::new();

    for frontend_root in frontend_roots() {
        visit(&root, &root.join(frontend_root), &allowlist, &mut violations)?;
    }
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("{violation}");
        }
        anyhow::bail!("backend boundary check failed");
    }

    Ok(())
}

fn visit(
    root: &Path,
    dir: &Path,
    allowlist: &HashSet<String>,
    violations: &mut Vec<String>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            visit(root, &path, allowlist, violations)?;
            continue;
        }

        let rel = path.strip_prefix(root)?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if !is_frontend_path(rel) || allowlist.contains(&rel_str) {
            continue;
        }

        let body = fs::read_to_string(&path).unwrap_or_default();
        for forbidden in FORBIDDEN {
            if body.contains(forbidden) {
                violations.push(format!("{rel_str} contains forbidden token `{forbidden}`"));
            }
        }
    }

    Ok(())
}

fn frontend_roots() -> [&'static Path; 3] {
    [
        Path::new("src"),
        Path::new("pyo3-ffi/src"),
        Path::new("pyo3-macros-backend/src"),
    ]
}

fn should_skip_dir(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::Normal(name) if name == "backend" || name == "tests" || name == "target"
        )
    })
}
