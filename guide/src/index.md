# The PyO3 user guide

Welcome to the PyO3 user guide! This book is a companion to [PyO3's API docs](https://docs.rs/pyo3). It contains examples and documentation to explain all of PyO3's use cases in detail.

The rough order of material in this user guide is as follows:
  1. Getting started
  2. Wrapping Rust code for use from Python
  3. How to use Python code from Rust
  4. Remaining topics which go into advanced concepts in detail

Please choose from the chapters on the left to jump to individual topics, or continue below to start with PyO3's README.

<div class="warning">

⚠️ Warning: API update in progress 🛠️

PyO3 0.21 has introduced a significant new API, termed the "Bound" API after the new smart pointer `Bound<T>`.

While most of this guide has been updated to the new API, it is possible some stray references to the older "GIL Refs" API such as `&PyAny` remain.
</div>

<hr style="opacity:0.2">

{{#include ../../README.md}}
