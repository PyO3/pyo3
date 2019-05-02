use std::fs::remove_dir_all;
use std::process::Command;

#[test]
fn test_building_example_using_venv() {
    fn create_venv(name: &str) {
        Command::new("python")
            .args(&["-m", "venv", name])
            .status()
            .expect("failed to create virtual environment")
            .success();
    }

    fn remove_venv(name: &str) {
        remove_dir_all(name).expect("failed to remove virtual environment");
    }

    fn is_example_built_using_venv(venv_name: &str, example_name: &str) -> bool {
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&[
                    "/C",
                    format!(
                        "{0}\\Scripts\\activate.bat && \
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
                        ". {0}/bin/activate && \
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
    create_venv(virtual_environment_name);
    assert!(is_example_built_using_venv(
        virtual_environment_name,
        "word-count"
    ));
    remove_venv(virtual_environment_name);
}
