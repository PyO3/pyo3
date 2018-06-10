import sys

from setuptools import setup
from setuptools.command.test import test as TestCommand

try:
    from setuptools_rust import RustExtension
except ImportError:
    import subprocess
    errno = subprocess.call([sys.executable, '-m', 'pip', 'install', 'setuptools-rust'])
    if errno:
        print("Please install setuptools-rust package")
        raise SystemExit(errno)
    else:
        from setuptools_rust import RustExtension

setup_requires = ['setuptools-rust>=0.6.1']
install_requires = []

setup(
    name='my-class',
    version='0.1.0',
    classifiers=[],
    packages=['my_class'],
    rust_extensions=[RustExtension('my_class._my_class', 'Cargo.toml')],
    install_requires=install_requires,
    setup_requires=setup_requires,
    include_package_data=True,
    zip_safe=False,
)
