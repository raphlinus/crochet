use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use druid::{AppLauncher, Widget, WindowDesc};

use crochet::{self, AppHolder, Button, DruidAppData, Label};

mod safe_ref;

use safe_ref::SafeRef;

struct PyAppLogic {
    py_app: PyObject,
}

impl PyAppLogic {
    fn run(&mut self, cx: &mut crochet::Cx) {
        // We need the transmute here because of the lifetime parameter.
        // The Cx itself is protected.
        let cx = unsafe { std::mem::transmute(cx) };
        Python::with_gil(|py| {
            SafeRef::scoped(py, cx, |cx_ref| {
                let py_cx = Py::new(py, Cx { inner: cx_ref }).unwrap();
                self.py_app.call(py, (py_cx,), None).unwrap();
            });
        });

    }
}

#[pyfunction]
fn pop_up_window(py_app: PyObject) -> PyResult<()> {
    let main_window = WindowDesc::new(|| ui_builder(py_app));
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .unwrap();
    Ok(())
}

fn ui_builder(py_app: PyObject) -> impl Widget<DruidAppData> {
    let mut app_logic = PyAppLogic { py_app };

    AppHolder::new(move |cx| app_logic.run(cx))
}

#[pyclass]
struct Cx {
    inner: SafeRef<crochet::Cx<'static>>,
}

#[pymethods]
impl Cx {
    fn label(&mut self, py: Python<'_>, text: &str) {
        if let Some(cx) = self.inner.try_get_mut(py) {
            Label::new(text).build(cx);
        }
    }

    fn button(&mut self, py: Python<'_>, text: &str) -> bool {
        if let Some(cx) = self.inner.try_get_mut(py) {
            Button::new(text).build(cx)
        } else {
            false
        }
    }
}

#[pymodule]
fn crochet_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pop_up_window, m)?)?;

    Ok(())
}
