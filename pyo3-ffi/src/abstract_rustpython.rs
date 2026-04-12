use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
#[cfg(any(Py_3_12, not(Py_LIMITED_API)))]
use libc::size_t;
use rustpython_vm::function::{FuncArgs, KwArgs};
use rustpython_vm::{AsObject, PyObjectRef};
use std::ffi::{c_char, c_int};

fn build_func_args(
    args: *mut PyObject,
    kwargs: *mut PyObject,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<FuncArgs> {
    let positional = if args.is_null() {
        Vec::new()
    } else {
        let args_obj = unsafe { ptr_to_pyobject_ref_borrowed(args) };
        args_obj
            .try_into_value::<rustpython_vm::builtins::PyTupleRef>(vm)
            .map(|tuple| tuple.as_slice().to_vec())
            .map_err(|_| vm.new_type_error("expected tuple args"))?
    };

    let mut kw = KwArgs::default();
    if !kwargs.is_null() {
        let kwargs_obj = unsafe { ptr_to_pyobject_ref_borrowed(kwargs) };
        let kwargs_dict = kwargs_obj
            .try_into_value::<rustpython_vm::builtins::PyDictRef>(vm)
            .map_err(|_| vm.new_type_error("expected dict kwargs"))?;
        for (k, v) in &kwargs_dict {
            let key = k
                .str(vm)
                .map_err(|_| vm.new_type_error("keywords must be strings"))?;
            kw = std::iter::once((key.as_str().to_owned(), v))
                .chain(kw)
                .collect();
        }
    }

    Ok(FuncArgs::new(positional, kw))
}

#[cfg(any(Py_3_12, not(Py_LIMITED_API)))]
pub const PY_VECTORCALL_ARGUMENTS_OFFSET: size_t =
    1 << (8 * std::mem::size_of::<size_t>() as size_t - 1);

#[inline]
pub unsafe fn PyObject_DelAttrString(o: *mut PyObject, attr_name: *const c_char) -> c_int {
    PyObject_SetAttrString(o, attr_name, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyObject_DelAttr(o: *mut PyObject, attr_name: *mut PyObject) -> c_int {
    PyObject_SetAttr(o, attr_name, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyObject_CallNoArgs(func: *mut PyObject) -> *mut PyObject {
    PyObject_CallObject(func, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyObject_Call(
    callable_object: *mut PyObject,
    args: *mut PyObject,
    kw: *mut PyObject,
) -> *mut PyObject {
    if callable_object.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let callable = unsafe { ptr_to_pyobject_ref_borrowed(callable_object) };
        let args = match build_func_args(args, kw, vm) {
            Ok(args) => args,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        match callable.call_with_args(args, vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyObject_CallObject(
    callable_object: *mut PyObject,
    args: *mut PyObject,
) -> *mut PyObject {
    PyObject_Call(callable_object, args, std::ptr::null_mut())
}

unsafe extern "C" {
    pub fn PyObject_CallMethodObjArgs(
        o: *mut PyObject,
        method: *mut PyObject,
        ...
    ) -> *mut PyObject;
}

#[cfg(any(Py_3_12, all(Py_3_11, not(Py_LIMITED_API))))]
#[inline]
pub unsafe fn PyObject_Vectorcall(
    _callable: *mut PyObject,
    _args: *const *mut PyObject,
    _nargsf: size_t,
    _kwnames: *mut PyObject,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(any(Py_3_12, all(Py_3_9, not(any(Py_LIMITED_API, PyPy)))))]
#[inline]
pub unsafe fn PyObject_VectorcallMethod(
    _name: *mut PyObject,
    _args: *const *mut PyObject,
    _nargsf: size_t,
    _kwnames: *mut PyObject,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(any(Py_3_12, all(Py_3_9, not(any(Py_LIMITED_API, PyPy)))))]
#[inline]
pub unsafe fn PyObject_CallMethodOneArg(
    _obj: *mut PyObject,
    _name: *mut PyObject,
    _arg: *mut PyObject,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(any(Py_3_12, all(Py_3_11, not(Py_LIMITED_API))))]
#[inline]
pub unsafe fn PyObject_VectorcallDict(
    _callable: *mut PyObject,
    _args: *const *mut PyObject,
    _nargsf: size_t,
    _kwargs: *mut PyObject,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(any(Py_3_12, all(Py_3_10, not(any(Py_LIMITED_API, PyPy)))))]
#[inline]
pub unsafe fn PyObject_CallOneArg(
    _callable: *mut PyObject,
    _arg: *mut PyObject,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_Type(o: *mut PyObject) -> *mut PyObject {
    Py_TYPE(o) as *mut PyObject
}

#[inline]
pub unsafe fn PyObject_Size(_o: *mut PyObject) -> Py_ssize_t {
    -1
}

#[inline]
pub unsafe fn PyObject_Length(o: *mut PyObject) -> Py_ssize_t {
    PyObject_Size(o)
}

#[inline]
pub unsafe fn PyObject_GetItem(_o: *mut PyObject, _key: *mut PyObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_SetItem(
    o: *mut PyObject,
    key: *mut PyObject,
    v: *mut PyObject,
) -> c_int {
    if o.is_null() || key.is_null() || v.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let key_obj = ptr_to_pyobject_ref_borrowed(key);
    let value_obj = ptr_to_pyobject_ref_borrowed(v);
    rustpython_runtime::with_vm(|vm| {
        let result = if let Some(f) = obj.mapping_unchecked().slots().ass_subscript.load() {
            f(obj.mapping_unchecked(), &key_obj, Some(value_obj), vm)
        } else if let Some(f) = obj.sequence_unchecked().slots().ass_item.load() {
            match key_obj
                .try_index(vm)
                .and_then(|i| i.try_to_primitive::<isize>(vm))
            {
                Ok(i) => f(obj.sequence_unchecked(), i, Some(value_obj), vm),
                Err(exc) => Err(exc),
            }
        } else {
            Err(vm.new_type_error(format!(
                "'{}' does not support item assignment",
                obj.class()
            )))
        };
        match result {
            Ok(()) => 0,
            Err(exc) => {
                set_vm_exception(exc);
                -1
            }
        }
    })
}

#[inline]
pub unsafe fn PyObject_DelItemString(_o: *mut PyObject, _key: *const c_char) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyObject_DelItem(_o: *mut PyObject, _key: *mut PyObject) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyObject_GetIter(_obj: *mut PyObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyIter_Next(_obj: *mut PyObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyNumber_Index(o: *mut PyObject) -> *mut PyObject {
    o
}

#[inline]
pub unsafe fn PySequence_Size(_o: *mut PyObject) -> Py_ssize_t {
    if _o.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(_o);
    rustpython_runtime::with_vm(|vm| match obj.length(vm) {
        Ok(len) => len as Py_ssize_t,
        Err(_) => -1,
    })
}

#[inline]
pub unsafe fn PyMapping_Size(_o: *mut PyObject) -> Py_ssize_t {
    -1
}

#[inline]
pub unsafe fn PyObject_LengthHint(o: *mut PyObject, default_value: Py_ssize_t) -> Py_ssize_t {
    if o.is_null() {
        return default_value;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| match obj.length_hint(default_value as usize, vm) {
        Ok(len) => len as Py_ssize_t,
        Err(_) => -1,
    })
}

#[inline]
pub unsafe fn PyIter_Check(obj: *mut PyObject) -> c_int {
    if obj.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(obj);
    rustpython_runtime::with_vm(|_vm| obj.class().slots.iternext.load().is_some().into())
}

#[inline]
pub unsafe fn PySequence_Check(o: *mut PyObject) -> c_int {
    if o.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    obj.sequence_unchecked().check().into()
}

#[inline]
pub unsafe fn PySequence_Concat(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    if o1.is_null() || o2.is_null() {
        return std::ptr::null_mut();
    }
    let lhs = ptr_to_pyobject_ref_borrowed(o1);
    let rhs = ptr_to_pyobject_ref_borrowed(o2);
    rustpython_runtime::with_vm(|vm| {
        lhs.sequence_unchecked()
            .concat(rhs.as_object(), vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_Repeat(o: *mut PyObject, count: Py_ssize_t) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .repeat(count as isize, vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_InPlaceConcat(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    if o1.is_null() || o2.is_null() {
        return std::ptr::null_mut();
    }
    let lhs = ptr_to_pyobject_ref_borrowed(o1);
    let rhs = ptr_to_pyobject_ref_borrowed(o2);
    rustpython_runtime::with_vm(|vm| {
        lhs.sequence_unchecked()
            .inplace_concat(rhs.as_object(), vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_InPlaceRepeat(o: *mut PyObject, count: Py_ssize_t) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .inplace_repeat(count as isize, vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_GetItem(o: *mut PyObject, index: Py_ssize_t) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .get_item(index as isize, vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_GetSlice(
    o: *mut PyObject,
    begin: Py_ssize_t,
    end: Py_ssize_t,
) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .get_slice(begin as isize, end as isize, vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_SetItem(
    o: *mut PyObject,
    index: Py_ssize_t,
    value: *mut PyObject,
) -> c_int {
    if o.is_null() || value.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let value = ptr_to_pyobject_ref_borrowed(value);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .set_item(index as isize, value, vm)
            .map(|()| 0)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_DelItem(o: *mut PyObject, index: Py_ssize_t) -> c_int {
    if o.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .del_item(index as isize, vm)
            .map(|()| 0)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_SetSlice(
    o: *mut PyObject,
    begin: Py_ssize_t,
    end: Py_ssize_t,
    value: *mut PyObject,
) -> c_int {
    if o.is_null() || value.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let value = ptr_to_pyobject_ref_borrowed(value);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .set_slice(begin as isize, end as isize, value, vm)
            .map(|()| 0)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_DelSlice(
    o: *mut PyObject,
    begin: Py_ssize_t,
    end: Py_ssize_t,
) -> c_int {
    if o.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .del_slice(begin as isize, end as isize, vm)
            .map(|()| 0)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_Count(o: *mut PyObject, value: *mut PyObject) -> Py_ssize_t {
    if o.is_null() || value.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let value = ptr_to_pyobject_ref_borrowed(value);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .count(value.as_object(), vm)
            .map(|count| count as Py_ssize_t)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_Contains(o: *mut PyObject, value: *mut PyObject) -> c_int {
    if o.is_null() || value.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let value = ptr_to_pyobject_ref_borrowed(value);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .contains(value.as_object(), vm)
            .map(|contains| contains as c_int)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_Index(o: *mut PyObject, value: *mut PyObject) -> Py_ssize_t {
    if o.is_null() || value.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    let value = ptr_to_pyobject_ref_borrowed(value);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .index(value.as_object(), vm)
            .map(|index| index as Py_ssize_t)
            .unwrap_or(-1)
    })
}

#[inline]
pub unsafe fn PySequence_List(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .list(vm)
            .map(|list| pyobject_ref_to_ptr(list.into()))
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PySequence_Tuple(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| {
        obj.sequence_unchecked()
            .tuple(vm)
            .map(|tuple| pyobject_ref_to_ptr(tuple.into()))
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyMapping_Keys(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&obj, "keys", ()) {
        Ok(value) => pyobject_ref_to_ptr(value),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyMapping_Values(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&obj, "values", ()) {
        Ok(value) => pyobject_ref_to_ptr(value),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyMapping_Items(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(o);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&obj, "items", ()) {
        Ok(value) => pyobject_ref_to_ptr(value),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
fn unary_number_op(
    o: *mut PyObject,
    op: impl FnOnce(&rustpython_vm::VirtualMachine, &rustpython_vm::PyObject) -> rustpython_vm::PyResult,
) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    let obj = unsafe { ptr_to_pyobject_ref_borrowed(o) };
    rustpython_runtime::with_vm(|vm| match op(vm, obj.as_object()) {
        Ok(result) => pyobject_ref_to_ptr(result),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
fn binary_number_op(
    o1: *mut PyObject,
    o2: *mut PyObject,
    op: impl FnOnce(&rustpython_vm::VirtualMachine, &rustpython_vm::PyObject, &rustpython_vm::PyObject) -> rustpython_vm::PyResult,
) -> *mut PyObject {
    if o1.is_null() || o2.is_null() {
        return std::ptr::null_mut();
    }
    let lhs = unsafe { ptr_to_pyobject_ref_borrowed(o1) };
    let rhs = unsafe { ptr_to_pyobject_ref_borrowed(o2) };
    rustpython_runtime::with_vm(|vm| match op(vm, lhs.as_object(), rhs.as_object()) {
        Ok(result) => pyobject_ref_to_ptr(result),
        Err(_) => std::ptr::null_mut(),
    })
}

#[inline]
pub unsafe fn PyNumber_Negative(o: *mut PyObject) -> *mut PyObject {
    unary_number_op(o, |vm, obj| vm._neg(obj))
}

#[inline]
pub unsafe fn PyNumber_Positive(o: *mut PyObject) -> *mut PyObject {
    unary_number_op(o, |vm, obj| vm._pos(obj))
}

#[inline]
pub unsafe fn PyNumber_Absolute(o: *mut PyObject) -> *mut PyObject {
    unary_number_op(o, |vm, obj| vm._abs(obj))
}

#[inline]
pub unsafe fn PyNumber_Invert(o: *mut PyObject) -> *mut PyObject {
    unary_number_op(o, |vm, obj| vm._invert(obj))
}

#[inline]
pub unsafe fn PyNumber_Add(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._add(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Subtract(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._sub(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Multiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._mul(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_MatrixMultiply(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._matmul(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_TrueDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._truediv(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_FloorDivide(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._floordiv(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Remainder(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._mod(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Lshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._lshift(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Rshift(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._rshift(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_And(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._and(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Or(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._or(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Xor(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._xor(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Divmod(o1: *mut PyObject, o2: *mut PyObject) -> *mut PyObject {
    binary_number_op(o1, o2, |vm, lhs, rhs| vm._divmod(lhs, rhs))
}

#[inline]
pub unsafe fn PyNumber_Power(
    o1: *mut PyObject,
    o2: *mut PyObject,
    o3: *mut PyObject,
) -> *mut PyObject {
    if o1.is_null() || o2.is_null() || o3.is_null() {
        return std::ptr::null_mut();
    }
    let lhs = ptr_to_pyobject_ref_borrowed(o1);
    let rhs = ptr_to_pyobject_ref_borrowed(o2);
    let mod_arg = ptr_to_pyobject_ref_borrowed(o3);
    rustpython_runtime::with_vm(|vm| match vm._pow(lhs.as_object(), rhs.as_object(), mod_arg.as_object()) {
        Ok(result) => pyobject_ref_to_ptr(result),
        Err(_) => std::ptr::null_mut(),
    })
}
