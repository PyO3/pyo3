from pyo3_pytests.buf_and_str import (
    BytesExtractor,
    return_memoryview,
    return_owned_memoryview,
)


def test_extract_bytes():
    extractor = BytesExtractor()
    message = b'\\(-"-;) A message written in bytes'
    assert extractor.from_bytes(message) == len(message)


def test_extract_str():
    extractor = BytesExtractor()
    message = '\\(-"-;) A message written as a string'
    assert extractor.from_str(message) == len(message)


def test_extract_str_lossy():
    extractor = BytesExtractor()
    message = '\\(-"-;) A message written with a trailing surrogate \ud800'
    rust_surrogate_len = extractor.from_str_lossy("\ud800")
    assert extractor.from_str_lossy(message) == len(message) - 1 + rust_surrogate_len


def test_extract_buffer():
    extractor = BytesExtractor()
    message = b'\\(-"-;) A message written in bytes'
    assert extractor.from_buffer(message) == len(message)

    arr = bytearray(b'\\(-"-;) A message written in bytes')
    assert extractor.from_buffer(arr) == len(arr)


def test_return_memoryview():
    view = return_memoryview()
    assert view.readonly
    assert view.contiguous
    assert view.tobytes() == b"hello world"


def test_return_owned_memoryview():
    view = return_owned_memoryview()
    assert view.readonly
    assert view.contiguous
    assert view.tobytes() == b"owned buffer data"
    assert len(view) == len(b"owned buffer data")


def test_owned_memoryview_keeps_data_alive():
    """Ensure the memoryview keeps the owner alive even after Python-side references are dropped."""
    view = return_owned_memoryview()
    # Access the data multiple times to ensure it's still valid
    assert view.tobytes() == b"owned buffer data"
    assert view[0] == ord(b"o")
    assert view[-1] == ord(b"a")
