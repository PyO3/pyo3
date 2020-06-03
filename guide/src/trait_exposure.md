# Expose a trait to Python

PyO3 allows you to expose code for which the argument type can be converted into a Python type.

However, how can you expose rust code that requires as argument a given trait implementation ?

Why would you do such a thing ?

### Pros
- Make your code available to Python users
- Code your complex logics with the help of the borrow checker

### Cons
- Not as fast as native Rust (type conversion has to be performed and one part of the code runs in Python)
- You need to adapt your code to expose it

## Example

Let's work with the following toy example of an implementation of a optimization solver operating on a given model.

Let's say we have a function `solve` that operates on a model and mutates it states.
The argument of the function can be any model that implement the `Model` trait :

```rust
pub trait Model {
  fn set_iteratives(&mut self, inputs: &Vec<f64>);
  fn compute(&mut self);
  fn get_results(&self) -> Vec<f64>;
}

pub fn solve<T: Model>(model: &mut T) {
  println!("Magic solver that mutates the model into a resolved state");
}
```

You cannot change that code as it runs on many Rust models.
You also have many Python models that cannot be solved as your solver is not available in that language.
Rewriting it in Python would be cumbersome and error-prone, as everything is already available in Rust.
How could we expose your solver to Python thanks to PyO3 ?

## Expose the required trait model

If you add a Python model implementing the same three methods as the trait, it seems it could be adapted to use your solver.
However, you cannot pass a `PyObject` to your solver as it does not implement the Rust trait (even if the Python model has the required methods)

You need to write a wrapper around your Python object in order to implement the trait.
This wrapper will call the Python model from Rust.
The methods signatures must be the same as your trait, that you cannot change.

The Python model you want to expose is the following one, it has all the required methods:

```python
class Model:
    def set_variables(self, inputs):
        self.inputs = inputs
    def compute(self):
        self.results = [elt**2 - 3 for elt in self.inputs]
    def get_results(self):
        return self.results
```

This wrapper will call the Python model from Rust, it is using a struct to hold the model as a `PyAny` object:

```rust
use pyo3::prelude::*;
use pyo3::types::PyAny;

# pub trait Model {
#   fn set_iteratives(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }

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

Now that this bit is implemented, let's expose the model wrapper to Python.
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

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyAny;
#
# pub trait Model {
#   fn set_iteratives(&mut self, inputs: &Vec<f64>);
#   fn compute(&mut self);
#   fn get_results(&self) -> Vec<f64>;
# }
#
# #[pyclass]
# struct UserModel {
#     model: Py<PyAny>,
# }
#
# #[pymodule]
# fn trait_exposure(_py: Python, m: &PyModule) -> PyResult<()> {
#    m.add_class::<UserModel>()?;
#    Ok(())
# }
#
# #[pymethods]
# impl UserModel {
#     #[new]
#     pub fn new(model: Py<PyAny>) -> Self {
#         UserModel { model }
#     }
# }
#
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

That's a bummer!
However, we can write a wrapper around this functions to call them directly.
These wrapper will also perform the types conversion in-between Python and Rust.

```rust
#[pymethods]
impl UserModel {

    pub fn set_variables_in_rust(&mut self, var: Vec<f64>) -> PyResult<()> {
        println!("Set variables from Python calling Rust");
        self.set_variables(&var);
        Ok(())
    }

    pub fn get_results_in_rust(&mut self) -> PyResult<Vec<f64>> {
        println!("Get results from Python calling Rust");
        let results = self.get_results();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_results = results.into_py(py);
        Ok(py_results)
    }

    pub fn compute_in_rust(&mut self) -> PyResult<()> {
        println!("Compute from Python calling Rust");
        self.compute();
        Ok(())
    }
}
```

This wrapper handles the type conversion between the PyO3 requirements and your trait, especially:
- the return type that must be a `PyResult`
- the references that are required by the trait but that cannot be passed from Python

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
  my_rust_model.set_variables_in_rust([2.0])
  print("Print value from Python: ", myModel.inputs)
  my_rust_model.compute_in_rust()
  print("Print value from Python: ", my_rust_model.get_results_in_rust())

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
Print value from Python:  [1.0]
```

We have succeed to expose a Rust model that implement the `Model` trait to Python!

We will expose the `solve` function, but before, let's talk about types errors.

## Type errors in Python

What happens if you have a type errors when using Python and how can you improve the error messages ?


### Wrong types in Python function arguments

Let's assume in the first case that you will use in your Python file `my_rust_model.set_variables_in_rust(2.0)` instead of `my_rust_model.set_variables_in_rust([2.0])`.

The Rust signature expect a vector, which is a list in Python.
What happens if instead of a vector, we pass a single value ?

At the execution of Python, we get :

```block
File "main.py", line 15, in <module>
   my_rust_model.set_variables_in_rust(2)
TypeError
```

It is a type error and Python points to it, so no worry.

### Wrong types in Python function signature

Let's assume now that the return type of one of the methods of our Model class is wrong, for example the get_results that is expected to be a `Vec<f64>` in Rust, so a list in Python.

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

This error code is quite not helpful for a Python user that does not know anything about Rust.

However, as being responsible for making the Rust code available to Python, we can do something about it.

The issue is that we called unwrap anywhere we could, and any panic from PyO3 will be directely forwarded to the end user.

Let's modify the code performing the type conversion to give a helpful error message to the Python user:

We used in our `get_results` method the following call that performs the extraction:

```rust
impl Model for UserModel {
  fn get_results(&self) -> Vec<f64> {
    println!("Get results from Rust calling Python");
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
```

Let's break it down in order to perform a better error handling:

```rust
impl Model for UserModel {
  fn get_results(&self) -> Vec<f64> {
      println!("Get results from Rust calling Python");
      let gil = Python::acquire_gil();
      let py = gil.python();
      let py_result: &PyAny = self
          .model
          .as_ref(py)
          .call_method("get_results", (), None)
          .unwrap();

      if py_result.get_type().name() != "list" {
          panic!("Expected a list for the get_results() method signature, got {}", py_result.get_type().name());
      }
      py_result.extract().unwrap()
  }
}
```

By doing so, you catch the result of the Python computation and check its type in order to be able to deliver a better error message before performing the unwrapping.

Of course, it does not cover all the possible wrong outputs:
the user could return a list of strings instead of a list of floats.

A runtime panic would still occurs thanks to PyO3, but with a error message hard to decipher for non-rust user.

It is up to the developer exposing the rust code to decide how much effort to invest into Python type errors handling.

## The final code

Now let's also expose the `solve()` function to make it available from Python.

It is not possible to expose direcly the `solve` function to Python, as the type conversion cannot be performed.

However, we can write a function wrapper that takes as argument a `UserModel`, which struct has also been expose to Python to call the core function `solve`.

It is also required to make the struct public.

```rust
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyAny;

pub trait Model {
    fn set_variables(&mut self, var: &Vec<f64>);
    fn get_results(&self) -> Vec<f64>;
    fn compute(&mut self);
}

pub fn solve<T: Model>(model: &mut T) {
  println!("Magic solver that mutates the model into a resolved state");
}

#[pyfunction]
pub fn solve_wrapper(model: &mut UserModel) {
    solve(model);
}

#[pyclass]
pub struct UserModel {
    model: Py<PyAny>,
}

#[pymodule]
fn trait_exposure(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UserModel>()?;
    m.add_wrapped(wrap_pyfunction!(solve_wrapper)).unwrap();
    Ok(())
}

#[pymethods]
impl UserModel {
    #[new]
    pub fn new(model: Py<PyAny>) -> Self {
        UserModel { model }
    }
    //    #[classmethod]
    pub fn set_variables_in_rust(&mut self, var: Vec<f64>) -> PyResult<()> {
        println!("Set variables from Python calling Rust");
        self.set_variables(&var);
        Ok(())
    }

    pub fn get_results_in_rust(&mut self) -> PyResult<Vec<f64>> {
        println!("Get results from Python calling Rust");
        let results = self.get_results();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_results = results.into_py(py);
        Ok(py_results)
    }

    pub fn compute_in_rust(&mut self) -> PyResult<()> {
        println!("Compute from Python calling Rust");
        self.compute();
        Ok(())
    }
}

impl Model for UserModel {
    fn set_variables(&mut self, var: &Vec<f64>) {
        println!("Set variables from Rust calling Python");
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
        println!("Get results from Rust calling Python");
        let gil = Python::acquire_gil();
        let py = gil.python();
        let py_result: &PyAny = self
            .model
            .as_ref(py)
            .call_method("get_results", (), None)
            .unwrap();

        if py_result.get_type().name() != "list" {
            panic!("Expected a list for the get_results() method signature, got {}", py_result.get_type().name());
        }
        py_result.extract().unwrap()
    }

    fn compute(&mut self) {
        println!("Compute from Rust calling Python");
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.model
            .as_ref(py)
            .call_method("compute", (), None)
            .unwrap();
    }
}
```
