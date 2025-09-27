# Using in Python a Rust function with trait bounds

PyO3 allows for easy conversion from Rust to Python for certain functions and classes (see the [conversion table](conversions/tables.md)).
However, it is not always straightforward to convert Rust code that requires a given trait implementation as an argument.

This tutorial explains how to convert a Rust function that takes a trait as argument for use in Python with classes implementing the same methods as the trait.

Why is this useful?

### Pros
- Make your Rust code available to Python users
- Code complex algorithms in Rust with the help of the borrow checker

### Cons
- Not as fast as native Rust (type conversion has to be performed and one part of the code runs in Python)
- You need to adapt your code to expose it

## Example

Let's work with the following basic example of an implementation of a optimization solver operating on a given model.

Let's say we have a function `solve` that operates on a model and mutates its state.
The argument of the function can be any model that implements the `Model` trait :

```rust,no_run
# #![allow(dead_code)]
pub trait Model {
    fn set_variables(&mut self, inputs: &Vec<f64>);
    fn compute(&mut self);
    fn get_results(&self) -> Vec<f64>;
}

pub fn solve<T: Model>(model: &mut T) {
    println!("Magic solver that mutates the model into a resolved state");
}
```
Let's assume we have the following constraints:
- We cannot change that code as it runs on many Rust models.
- We also have many Python models that cannot be solved as this solver is not available in that language.
Rewriting it in Python would be cumbersome and error-prone, as everything is already available in Rust.

How could we expose this solver to Python thanks to PyO3 ?

## Implementation of the trait bounds for the Python class

If a Python class implements the same three methods as the `Model` trait, it seems logical it could be adapted to use the solver.
However, it is not possible to pass a `Py<PyAny>` to it as it does not implement the Rust trait (even if the Python model has the required methods).

In order to implement the trait, we must write a wrapper around the calls in Rust to the Python model.
The method signatures must be the same as the trait, keeping in mind that the Rust trait cannot be changed for the purpose of making the code available in Python.

The Python model we want to expose is the following one, which already contains all the required methods:

```python
class Model:
    def set_variables(self, inputs):
        self.inputs = inputs
    def compute(self):
        self.results = [elt**2 - 3 for elt in self.inputs]
    def get_results(self):
        return self.results
```

The following wrapper will call the Python model from Rust, using a struct to hold the model as a `PyAny` object:

```rust,no_run
# #![allow(dead_code)]
use pyo3::prelude::*;
use pyo3::types::PyList;

# pub trait Model {
#   fn set_variables(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }

struct UserModel {
    model: Py<PyAny>,
}

impl Model for UserModel {
    fn set_variables(&mut self, var: &Vec<f64>) {
        println!("Rust calling Python to set the variables");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("set_variables", (PyList::new(py, var).unwrap(),), None)
                .unwrap();
        })
    }

    fn get_results(&self) -> Vec<f64> {
        println!("Rust calling Python to get the results");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("get_results", (), None)
                .unwrap()
                .extract()
                .unwrap()
        })
    }

    fn compute(&mut self) {
        println!("Rust calling Python to perform the computation");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("compute", (), None)
                .unwrap();
        })
    }
}
```

Now that this bit is implemented, let's expose the model wrapper to Python.
Let's add the PyO3 annotations and add a constructor:

```rust,no_run
# #![allow(dead_code)]
# fn main() {}
# pub trait Model {
#   fn set_variables(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
# use pyo3::prelude::*;

#[pyclass]
struct UserModel {
    model: Py<PyAny>,
}

#[pymethods]
impl UserModel {
    #[new]
    pub fn new(model: Py<PyAny>) -> Self {
        UserModel { model }
    }
}

#[pymodule]
mod trait_exposure {
    #[pymodule_export]
    use super::UserModel;
}
```

Now we add the PyO3 annotations to the trait implementation:

```rust,ignore
#[pymethods]
impl Model for UserModel {
    // the previous trait implementation
}
```

However, the previous code will not compile. The compilation error is the following one:
`error: #[pymethods] cannot be used on trait impl blocks`

That's a bummer!
However, we can write a second wrapper around these functions to call them directly.
This wrapper will also perform the type conversions between Python and Rust.

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;
#
# pub trait Model {
#   fn set_variables(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
#
# #[pyclass]
# struct UserModel {
#     model: Py<PyAny>,
# }
#
# impl Model for UserModel {
#  fn set_variables(&mut self, var: &Vec<f64>) {
#      println!("Rust calling Python to set the variables");
#      Python::attach(|py| {
#          self.model.bind(py)
#              .call_method("set_variables", (PyList::new(py, var).unwrap(),), None)
#              .unwrap();
#      })
#  }
#
#  fn get_results(&self) -> Vec<f64> {
#      println!("Rust calling Python to get the results");
#      Python::attach(|py| {
#          self.model
#              .bind(py)
#              .call_method("get_results", (), None)
#              .unwrap()
#              .extract()
#              .unwrap()
#      })
#  }
#
#  fn compute(&mut self) {
#      println!("Rust calling Python to perform the computation");
#      Python::attach(|py| {
#          self.model
#              .bind(py)
#              .call_method("compute", (), None)
#              .unwrap();
#      })
#
#  }
# }

#[pymethods]
impl UserModel {
    pub fn set_variables(&mut self, var: Vec<f64>) {
        println!("Set variables from Python calling Rust");
        Model::set_variables(self, &var)
    }

    pub fn get_results(&mut self) -> Vec<f64> {
        println!("Get results from Python calling Rust");
        Model::get_results(self)
    }

    pub fn compute(&mut self) {
        println!("Compute from Python calling Rust");
        Model::compute(self)
    }
}
```
This wrapper handles the type conversion between the PyO3 requirements and the trait.
In order to meet PyO3 requirements, this wrapper must:
- return an object of type `PyResult`
- use only values, not references in the method signatures

Let's run the file python file:

```python
class Model:
    def set_variables(self, inputs):
        self.inputs = inputs
    def compute(self):
        self.results = [elt**2 - 3 for elt in self.inputs]
    def get_results(self):
        return self.results

if __name__=="__main__":
  import trait_exposure

  myModel = Model()
  my_rust_model = trait_exposure.UserModel(myModel)
  my_rust_model.set_variables([2.0])
  print("Print value from Python: ", myModel.inputs)
  my_rust_model.compute()
  print("Print value from Python through Rust: ", my_rust_model.get_results())
  print("Print value directly from Python: ", myModel.get_results())
```

This outputs:

```block
Set variables from Python calling Rust
Set variables from Rust calling Python
Print value from Python:  [2.0]
Compute from Python calling Rust
Compute from Rust calling Python
Get results from Python calling Rust
Get results from Rust calling Python
Print value from Python through Rust:  [1.0]
Print value directly from Python:  [1.0]
```

We have now successfully exposed a Rust model that implements the `Model` trait to Python!

We will now expose the `solve` function, but before, let's talk about types errors.

## Type errors in Python

What happens if you have type errors when using Python and how can you improve the error messages?


### Wrong types in Python function arguments

Let's assume in the first case that you will use in your Python file `my_rust_model.set_variables(2.0)` instead of `my_rust_model.set_variables([2.0])`.

The Rust signature expects a vector, which corresponds to a list in Python.
What happens if instead of a vector, we pass a single value ?

At the execution of Python, we get :

```block
File "main.py", line 15, in <module>
   my_rust_model.set_variables(2)
TypeError
```

It is a type error and Python points to it, so it's easy to identify and solve.

### Wrong types in Python method signatures

Let's assume now that the return type of one of the methods of our Model class is wrong, for example the `get_results` method that is expected to return a `Vec<f64>` in Rust, a list in Python.

```python
class Model:
    def set_variables(self, inputs):
        self.inputs = inputs
    def compute(self):
        self.results = [elt**2 -3 for elt in self.inputs]
    def get_results(self):
        return self.results[0]
        #return self.results <-- this is the expected output
```

This call results in the following panic:

```block
pyo3_runtime.PanicException: called `Result::unwrap()` on an `Err` value: PyErr { type: Py(0x10dcf79f0, PhantomData) }
```

This error code is not helpful for a Python user that does not know anything about Rust, or someone that does not know PyO3 was used to interface the Rust code.

However, as we are responsible for making the Rust code available to Python, we can do something about it.

The issue is that we called `unwrap` anywhere we could, and therefore any panic from PyO3 will be directly forwarded to the end user.

Let's modify the code performing the type conversion to give a helpful error message to the Python user:

We used in our `get_results` method the following call that performs the type conversion:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;
#
# pub trait Model {
#   fn set_variables(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
#
# #[pyclass]
# struct UserModel {
#     model: Py<PyAny>,
# }

impl Model for UserModel {
    fn get_results(&self) -> Vec<f64> {
        println!("Rust calling Python to get the results");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("get_results", (), None)
                .unwrap()
                .extract()
                .unwrap()
        })
    }
#     fn set_variables(&mut self, var: &Vec<f64>) {
#         println!("Rust calling Python to set the variables");
#         Python::attach(|py| {
#             self.model.bind(py)
#                 .call_method("set_variables", (PyList::new(py, var).unwrap(),), None)
#                 .unwrap();
#         })
#     }
#
#     fn compute(&mut self) {
#         println!("Rust calling Python to perform the computation");
#         Python::attach(|py| {
#             self.model
#                 .bind(py)
#                 .call_method("compute", (), None)
#                 .unwrap();
#         })
#     }
}
```

Let's break it down in order to perform better error handling:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;
#
# pub trait Model {
#   fn set_variables(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
#
# #[pyclass]
# struct UserModel {
#     model: Py<PyAny>,
# }

impl Model for UserModel {
    fn get_results(&self) -> Vec<f64> {
        println!("Get results from Rust calling Python");
        Python::attach(|py| {
            let py_result: Bound<'_, PyAny> = self
                .model
                .bind(py)
                .call_method("get_results", (), None)
                .unwrap();

            if py_result.get_type().name().unwrap() != "list" {
                panic!(
                    "Expected a list for the get_results() method signature, got {}",
                    py_result.get_type().name().unwrap()
                );
            }
            py_result.extract()
        })
        .unwrap()
    }
#     fn set_variables(&mut self, var: &Vec<f64>) {
#         println!("Rust calling Python to set the variables");
#         Python::attach(|py| {
#             let py_model = self.model.bind(py)
#                 .call_method("set_variables", (PyList::new(py, var).unwrap(),), None)
#                 .unwrap();
#         })
#     }
#
#     fn compute(&mut self) {
#         println!("Rust calling Python to perform the computation");
#         Python::attach(|py| {
#             self.model
#                 .bind(py)
#                 .call_method("compute", (), None)
#                 .unwrap();
#         })
#     }
}
```

By doing so, you catch the result of the Python computation and check its type in order to be able to deliver a better error message before performing the unwrapping.

Of course, it does not cover all the possible wrong outputs:
the user could return a list of strings instead of a list of floats.
In this case, a runtime panic would still occur due to PyO3, but with an error message much more difficult to decipher for non-rust user.

It is up to the developer exposing the rust code to decide how much effort to invest into Python type error handling and improved error messages.

## The final code

Now let's expose the `solve()` function to make it available from Python.

It is not possible to directly expose the `solve` function to Python, as the type conversion cannot be performed.
It requires an object implementing the `Model` trait as input.

However, the `UserModel` already implements this trait.
Because of this, we can write a function wrapper that takes the `UserModel`--which has already been exposed to Python--as an argument in order to call the core function `solve`.

It is also required to make the struct public.

```rust,no_run
# #![allow(dead_code)]
# fn main() {}
use pyo3::prelude::*;
use pyo3::types::PyList;

pub trait Model {
    fn set_variables(&mut self, var: &Vec<f64>);
    fn get_results(&self) -> Vec<f64>;
    fn compute(&mut self);
}

pub fn solve<T: Model>(model: &mut T) {
    println!("Magic solver that mutates the model into a resolved state");
}

#[pyfunction]
#[pyo3(name = "solve")]
pub fn solve_wrapper(model: &mut UserModel) {
    solve(model);
}

#[pyclass]
pub struct UserModel {
    model: Py<PyAny>,
}

#[pymethods]
impl UserModel {
    #[new]
    pub fn new(model: Py<PyAny>) -> Self {
        UserModel { model }
    }

    pub fn set_variables(&mut self, var: Vec<f64>) {
        println!("Set variables from Python calling Rust");
        Model::set_variables(self, &var)
    }

    pub fn get_results(&mut self) -> Vec<f64> {
        println!("Get results from Python calling Rust");
        Model::get_results(self)
    }

    pub fn compute(&mut self) {
        Model::compute(self)
    }
}

#[pymodule]
mod trait_exposure {
    #[pymodule_export]
    use super::{UserModel, solve_wrapper};
}

impl Model for UserModel {
    fn set_variables(&mut self, var: &Vec<f64>) {
        println!("Rust calling Python to set the variables");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("set_variables", (PyList::new(py, var).unwrap(),), None)
                .unwrap();
        })
    }

    fn get_results(&self) -> Vec<f64> {
        println!("Get results from Rust calling Python");
        Python::attach(|py| {
            let py_result: Bound<'_, PyAny> = self
                .model
                .bind(py)
                .call_method("get_results", (), None)
                .unwrap();

            if py_result.get_type().name().unwrap() != "list" {
                panic!(
                    "Expected a list for the get_results() method signature, got {}",
                    py_result.get_type().name().unwrap()
                );
            }
            py_result.extract()
        })
        .unwrap()
    }

    fn compute(&mut self) {
        println!("Rust calling Python to perform the computation");
        Python::attach(|py| {
            self.model
                .bind(py)
                .call_method("compute", (), None)
                .unwrap();
        })
    }
}
```
