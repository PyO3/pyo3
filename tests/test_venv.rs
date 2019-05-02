use std::fs::remove_dir_all;
use std::process::Command;

#[test]
fn test_building_example_using_venv() {
    fn is_example_built_using_venv(venv_name: &str, example_name: &str) -> bool {
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&[
                    "/C",
                    format!(
                        "python -m venv {0} && \
                         {0}\\Scripts\\activate.bat && \
                         cd examples\\{1} && \
                         python setup.py build",
                        venv_name, example_name
                    )
                    .as_str(),
                ])
                .status()
                .expect("failed to execute commands chain")
                .success()
        } else {
            Command::new("sh")
                .args(&[
                    "-c",
                    format!(
                        "python -m venv {0} && \
                         . {0}/bin/activate && \
                         cd examples/{1} && \
                         python setup.py build",
                        venv_name, example_name
                    )
                    .as_str(),
                ])
                .status()
                .expect("failed to execute commands chain")
                .success()
        }
    }

    let virtual_environment_name = "venv";
    assert!(is_example_built_using_venv(
        virtual_environment_name,
        "word-count"
    ));
    remove_dir_all(virtual_environment_name)
        .expect("failed to remove virtual environment directory");
}
