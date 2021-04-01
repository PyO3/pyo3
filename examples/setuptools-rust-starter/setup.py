from setuptools import setup
from setuptools_rust import RustExtension


setup(
    name="setuptools-rust-starter",
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
    packages=["setuptools_rust_starter"],
    rust_extensions=[
        RustExtension(
            "setuptools_rust_starter._setuptools_rust_starter",
            debug=False,
        ),
    ],
    include_package_data=True,
    zip_safe=False,
)
