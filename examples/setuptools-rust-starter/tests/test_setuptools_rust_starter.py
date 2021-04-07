from setuptools_rust_starter import PythonClass, ExampleClass


def test_python_class() -> None:
    py_class = PythonClass(value=10)
    assert py_class.value == 10


def test_example_class() -> None:
    example = ExampleClass(value=11)
    assert example.value == 11
