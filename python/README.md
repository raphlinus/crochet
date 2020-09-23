# Python bindings

This crate contains the beginnings of Python bindings for Druid with the Crochet architecture.

It uses [PyO3] for the Python integration. Please refer to their documentation for prerequisites and instructions how to get extension modules written in Rust to work with the Python on your machine. However, as a quick start, on Windows (using a mingw shell) the following command should work:

```sh
cargo build && cp target/debug/crochet_py.dll crochet_py.pyd && python run.py
```

Functionality is currently very limited; at the moment, this is really a proof of concept to see whether the integration is possible. To build out more of the Crochet architecture, we envision using [inspect.currentframe()] to give unique caller locations, comparable to `#[track_caller]` in Rust.

Also, it's likely the integration would use explicit `begin` and `end` methods across the language boundary, relying on Python's [`with`] to enforce nesting, rather than running through Rust closures. But these are details to be determined.

[PyO3]: https://github.com/PyO3/pyo3
[inspect.currentframe]: https://docs.python.org/3/library/inspect.html#inspect.currentframe
[`with`]: https://docs.python.org/2.5/whatsnew/pep-343.html
