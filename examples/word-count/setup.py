from setuptools import setup
from setuptools_rust import RustExtension

setup(
    rust_extensions=[RustExtension("word_count.word_count", debug=False)],
)
