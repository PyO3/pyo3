# Expose a trait to Python

PyO3 allows you to expose code for which the argument type can be converted into a Python type.

However, how can you expose rust code that requires as argument a given trait implementation ?

## The pros and cons

### Pros
- Make your code available to Python users
- Code your complex logics with the help of the borrow checker

### Cons
- Not as fast as native Rust (type conversion, one part of the code runs in Python)
- You need to adapt your code to expose it

## Example

Let's work with the following toy example of an implementation of the [Newton-Raphson Method](https://en.wikipedia.org/wiki/Newton%27s_method)

Let's say we have a function `solve` that operates on a model and mutates it states.
The argument of the function can be any model that implement the `Model` trait :

```rust
pub trait Model {
  fn set_iteratives(&mut self, inputs: &Vec<f64>);
  fn compute(&mut self);
  fn get_results(&self) -> Vec<f64>;
}

pub fn solve<T: Model>(&mut T) {
  // magic mutate the model so it is in a resolved state
}
```

You cannot change that code as it runs on many Rust models.
You also have many Python models that cannot be solved as your solver is not available in that language.
Rewriting it in Python would be cumbersome and error-prone, as everything is already available in Rust.
How could we expose your solver to Python thanks to PyO3 ?

## Expose the required trait model

If you add a Python model implementing the same three methods as the trait, it seems it could be adapted to use your solver.
However, you cannot pass a PyObject to your solver as it does not implement the Rust trait (even if the Python model has the required methods)

You need to write a wrapper around your Python object in order to implement the trait.
This wrapper will call the Python model from Rust.
The methods signatures must be the same as your trait, that you cannot change.

The Python model you want to expose is the following one, it has all the required methods:

```python
class Model:
    def set_variables(self, a):
        self.a = a
    def compute(self):
        self.b = [elt**2 for elt in self.a]
    def get_results(self):
        return [elt - 3 for elt in self.b]
```

This wrapper will call the Python model from Rust, it is using a struct to hold the model as a `PyAny` object:

```rust
# pub trait Model {
#   fn set_iteratives(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
use pyo3::prelude::*;
use pyo3::types::PyAny;

struct UserModel {
    model: Py<PyAny>,
}

impl Model for UserModel {
    fn set_variables(&mut self, var: &Vec<f64>) {
        println!("Rust calling Python to set the variables");
        let gil = Python::acquire_gil();
        let py = gil.python();
        let values: Vec<f64> = var.clone();
        let list: PyObject = values.into_py(py);
        let py_model = self.model.as_ref(py);
        py_model
            .call_method("set_variables", (list,), None)
            .unwrap();
    }

    fn get_results(&self) -> Vec<f64> {
        println!("Rust calling Python to get the results");
        let gil = Python::acquire_gil();
        let py = gil.python();
        self
            .model
            .as_ref(py)
            .call_method("get_results", (), None)
            .unwrap()
            .extract()
            .unwrap()
    }

    fn compute(&mut self) {
        println!("Rust calling Python to perform the computation");
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.model
            .as_ref(py)
            .call_method("compute", (), None)
            .unwrap();
    }
}
```

Now that this bit is implement it, let's expose the model wrapper to Python.
Let's add the PyO3 annotations and add a constructor:

```rust
# pub trait Model {
#   fn set_iteratives(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
# use pyo3::prelude::*;
# use pyo3::types::PyAny;

#[pyclass]
struct UserModel {
    model: Py<PyAny>,
}

#[pymodule]
fn trait_exposure(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UserModel>()?;
    Ok(())
}

#[pymethods]
impl UserModel {
    #[new]
    pub fn new(model: Py<PyAny>) -> Self {
        UserModel { model }
    }
}
```

Let's add the PyO3 annotations to the trait implementation:

```compile_fail
# pub trait Model {
#   fn set_iteratives(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
# use pyo3::prelude::*;
# use pyo3::types::PyAny;

# #[pyclass]
# struct UserModel {
#     model: Py<PyAny>,
# }

# #[pymodule]
# fn trait_exposure(_py: Python, m: &PyModule) -> PyResult<()> {
#    m.add_class::<UserModel>()?;
#    Ok(())
# }

# #[pymethods]
# impl UserModel {
#     #[new]
#     pub fn new(model: Py<PyAny>) -> Self {
#         UserModel { model }
#     }
# }

#[pymethods]
impl Model for UserModel {
  // the previous trait implementation
  # fn set_variables(&mut self, var: &Vec<f64>) {
  #     println!("Rust calling Python to set the variables");
  #     let gil = Python::acquire_gil();
  #     let py = gil.python();
  #     let values: Vec<f64> = var.clone();
  #     let list: PyObject = values.into_py(py);
  #     let py_model = self.model.as_ref(py);
  #     py_model
  #        .call_method("set_variables", (list,), None)
  #        .unwrap();
  # }
  #
  # fn get_results(&self) -> Vec<f64> {
  #     println!("Rust calling Python to get the results");
  #     let gil = Python::acquire_gil();
  #     let py = gil.python();
  #     self
  #         .model
  #         .as_ref(py)
  #         .call_method("get_results", (), None)
  #         .unwrap()
  #         .extract()
  #         .unwrap()
  # }
  #
  # fn compute(&mut self) {
  #     println!("Rust calling Python to perform the computation");
  #     let gil = Python::acquire_gil();
  #     let py = gil.python();
  #     self.model
  #         .as_ref(py)
  #         .call_method("compute", (), None)
  #         .unwrap();
  # }
}
```

You get the compilation error:
`error: #[pymethods] can not be used only with trait impl block`

That's a bummer! However, we can write a wrapper around this functions to call them directly.
These wrapper will also perform the types conversion in-between Python and Rust.


## Type errors in Python
