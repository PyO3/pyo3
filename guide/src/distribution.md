# Distribution

There are two way to distribute your module as python package: The old [setuptools-rust](https://github.com/PyO3/setuptools-rust) and the new [pyo3-pack](https://github.com/pyo3/pyo3-pack). setuptools-rust needs some configuration files (`setup.py`,  `MANIFEST.in`, `build-wheels.sh`, etc.) and external tools (docker, twine). pyo3-pack doesn't need any configuration files. It can not yet build sdist though ([pyo3/pyo3-pack#2](https://github.com/PyO3/pyo3-pack/issues/2)).
