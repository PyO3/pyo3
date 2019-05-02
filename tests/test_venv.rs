use std::fs::remove_dir_all;
use std::process::Command;

#[test]
fn test_building_example_using_venv() {
    fn is_example_built_using_venv(venv_name: &str, example_name: &str) -> bool {
        let commands_chain = if cfg!(target_os = "windows") {
            format!(
                "python -m venv {0} && \
                 {0}\\Scripts\\activate.bat && \
                 cd examples\\{1} && \
                 python setup.py build",
                venv_name, example_name
            );
        } else {
            format!(
                "python -m venv {0} && \
                 source {0}/bin/activate && \
                 cd examples/{1} && \
                 python setup.py build",
                venv_name, example_name
            );
        };
        Command::new(commands_chain)
            .status()
            .expect("failed to execute commands chain")
            .success()
    }

    let virtual_environment_name = "venv";
    assert!(is_example_built_using_venv(
        virtual_environment_name,
        "word-count"
    ));
    remove_dir_all(virtual_environment_name)
        .expect("failed to remove virtual environment directory");
}
