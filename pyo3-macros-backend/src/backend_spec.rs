use syn::Ident;

/// Backend-neutral description of a lowered `#[pyclass]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClassSpec {
    pub rust_ident: Ident,
    pub python_name: String,
    pub module: Option<String>,
    pub has_methods: bool,
}

impl ClassSpec {
    pub fn new(
        rust_ident: Ident,
        python_name: String,
        module: Option<String>,
        has_methods: bool,
    ) -> Self {
        Self {
            rust_ident,
            python_name,
            module,
            has_methods,
        }
    }
}

/// Backend-neutral description of a lowered `#[pymethods]` entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MethodSpec {
    pub python_name: String,
    pub is_classmethod: bool,
    pub is_staticmethod: bool,
    pub is_getter: bool,
    pub is_setter: bool,
    pub is_constructor: bool,
}

impl MethodSpec {
    pub fn new(
        python_name: String,
        is_classmethod: bool,
        is_staticmethod: bool,
        is_getter: bool,
        is_setter: bool,
        is_constructor: bool,
    ) -> Self {
        Self {
            python_name,
            is_classmethod,
            is_staticmethod,
            is_getter,
            is_setter,
            is_constructor,
        }
    }
}

/// Backend-neutral description of a lowered `#[pyfunction]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    pub python_name: String,
}

impl FunctionSpec {
    pub fn new(python_name: String) -> Self {
        Self { python_name }
    }
}
