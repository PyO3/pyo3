from setuptools import setup
from setuptools_rust import RustExtension


setup(
    name="pyo3-benchmarks",
    version="0.1.0",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python",
        "Programming Language :: Rust",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
    ],
    packages=["pyo3_benchmarks"],
    rust_extensions=[
        RustExtension(
            "pyo3_benchmarks._pyo3_benchmarks",
            debug=False,
        ),
    ],
    include_package_data=True,
    zip_safe=False,
)
