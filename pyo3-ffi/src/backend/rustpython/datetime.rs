use crate::datetime::PyDateTime_CAPI;
use crate::pyerrors::set_vm_exception;
use crate::{
    ptr_to_pyobject_ref_borrowed, pyobject_ref_as_ptr, pyobject_ref_to_ptr, rustpython_runtime,
    PyLong_AsLong, PyLong_Check, PyObject, PyObject_GetAttrString, PyTypeObject, Py_DECREF,
    Py_None,
};
use rustpython_vm::function::{FuncArgs, KwArgs};
use rustpython_vm::PyObjectRef;
use std::ffi::c_int;

fn import_datetime(
    vm: &rustpython_vm::VirtualMachine,
) -> Result<rustpython_vm::PyObjectRef, rustpython_vm::builtins::PyBaseExceptionRef> {
    let _ = vm.import("_operator", 0);
    let datetime = vm.import("datetime", 0)?;
    if datetime.get_attr("datetime_CAPI", vm).is_err() {
        let required_attrs = [
            "date",
            "datetime",
            "time",
            "timedelta",
            "tzinfo",
            "timezone",
        ];
        let has_runtime_surface = required_attrs
            .iter()
            .all(|name| datetime.get_attr(*name, vm).is_ok());
        if !has_runtime_surface {
            return Err(
                vm.new_attribute_error("module 'datetime' has no attribute 'datetime_CAPI'")
            );
        }
    }
    Ok(datetime)
}

#[inline]
unsafe fn get_attr(obj: *mut PyObject, field: &std::ffi::CStr) -> c_int {
    let result = PyObject_GetAttrString(obj, field.as_ptr());
    Py_DECREF(result);
    if PyLong_Check(result) == 1 {
        PyLong_AsLong(result) as c_int
    } else {
        0
    }
}

#[inline]
pub unsafe fn PyDateTime_GET_YEAR(o: *mut PyObject) -> c_int {
    get_attr(o, c"year")
}

#[inline]
pub unsafe fn PyDateTime_GET_MONTH(o: *mut PyObject) -> c_int {
    get_attr(o, c"month")
}

#[inline]
pub unsafe fn PyDateTime_GET_DAY(o: *mut PyObject) -> c_int {
    get_attr(o, c"day")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_HOUR(o: *mut PyObject) -> c_int {
    get_attr(o, c"hour")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_MINUTE(o: *mut PyObject) -> c_int {
    get_attr(o, c"minute")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_SECOND(o: *mut PyObject) -> c_int {
    get_attr(o, c"second")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    get_attr(o, c"microsecond")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_FOLD(o: *mut PyObject) -> c_int {
    get_attr(o, c"fold")
}

#[inline]
pub unsafe fn PyDateTime_DATE_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    let res = PyObject_GetAttrString(o, c"tzinfo".as_ptr().cast());
    Py_DECREF(res);
    res
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_HOUR(o: *mut PyObject) -> c_int {
    get_attr(o, c"hour")
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_MINUTE(o: *mut PyObject) -> c_int {
    get_attr(o, c"minute")
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_SECOND(o: *mut PyObject) -> c_int {
    get_attr(o, c"second")
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_MICROSECOND(o: *mut PyObject) -> c_int {
    get_attr(o, c"microsecond")
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_FOLD(o: *mut PyObject) -> c_int {
    get_attr(o, c"fold")
}

#[inline]
pub unsafe fn PyDateTime_TIME_GET_TZINFO(o: *mut PyObject) -> *mut PyObject {
    let res = PyObject_GetAttrString(o, c"tzinfo".as_ptr().cast());
    Py_DECREF(res);
    res
}

#[inline]
pub unsafe fn PyDateTime_DELTA_GET_DAYS(o: *mut PyObject) -> c_int {
    get_attr(o, c"days")
}

#[inline]
pub unsafe fn PyDateTime_DELTA_GET_SECONDS(o: *mut PyObject) -> c_int {
    get_attr(o, c"seconds")
}

#[inline]
pub unsafe fn PyDateTime_DELTA_GET_MICROSECONDS(o: *mut PyObject) -> c_int {
    get_attr(o, c"microseconds")
}

fn call_datetime_type(
    cls: *mut PyTypeObject,
    positional: Vec<PyObjectRef>,
    kwargs: KwArgs,
) -> *mut PyObject {
    if cls.is_null() {
        return std::ptr::null_mut();
    }
    let cls_obj = unsafe { ptr_to_pyobject_ref_borrowed(cls.cast()) };
    rustpython_runtime::with_vm(|vm| {
        match cls_obj.call_with_args(FuncArgs::new(positional, kwargs), vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn date_from_date(
    year: c_int,
    month: c_int,
    day: c_int,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        call_datetime_type(
            cls,
            vec![
                vm.ctx.new_int(year).into(),
                vm.ctx.new_int(month).into(),
                vm.ctx.new_int(day).into(),
            ],
            KwArgs::default(),
        )
    })
}

unsafe extern "C" fn datetime_from_date_and_time(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    tzinfo: *mut PyObject,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let mut positional = vec![
            vm.ctx.new_int(year).into(),
            vm.ctx.new_int(month).into(),
            vm.ctx.new_int(day).into(),
            vm.ctx.new_int(hour).into(),
            vm.ctx.new_int(minute).into(),
            vm.ctx.new_int(second).into(),
            vm.ctx.new_int(microsecond).into(),
        ];
        if !tzinfo.is_null() && unsafe { tzinfo != Py_None() } {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(tzinfo) });
        }
        call_datetime_type(cls, positional, KwArgs::default())
    })
}

unsafe extern "C" fn datetime_from_date_and_time_and_fold(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    tzinfo: *mut PyObject,
    fold: c_int,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let mut positional = vec![
            vm.ctx.new_int(year).into(),
            vm.ctx.new_int(month).into(),
            vm.ctx.new_int(day).into(),
            vm.ctx.new_int(hour).into(),
            vm.ctx.new_int(minute).into(),
            vm.ctx.new_int(second).into(),
            vm.ctx.new_int(microsecond).into(),
        ];
        if !tzinfo.is_null() && unsafe { tzinfo != Py_None() } {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(tzinfo) });
        }
        let kwargs = std::iter::once(("fold".to_owned(), vm.ctx.new_int(fold).into())).collect();
        call_datetime_type(cls, positional, kwargs)
    })
}

unsafe extern "C" fn time_from_time(
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    tzinfo: *mut PyObject,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let mut positional = vec![
            vm.ctx.new_int(hour).into(),
            vm.ctx.new_int(minute).into(),
            vm.ctx.new_int(second).into(),
            vm.ctx.new_int(microsecond).into(),
        ];
        if !tzinfo.is_null() && unsafe { tzinfo != Py_None() } {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(tzinfo) });
        }
        call_datetime_type(cls, positional, KwArgs::default())
    })
}

unsafe extern "C" fn time_from_time_and_fold(
    hour: c_int,
    minute: c_int,
    second: c_int,
    microsecond: c_int,
    tzinfo: *mut PyObject,
    fold: c_int,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let mut positional = vec![
            vm.ctx.new_int(hour).into(),
            vm.ctx.new_int(minute).into(),
            vm.ctx.new_int(second).into(),
            vm.ctx.new_int(microsecond).into(),
        ];
        if !tzinfo.is_null() && unsafe { tzinfo != Py_None() } {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(tzinfo) });
        }
        let kwargs = std::iter::once(("fold".to_owned(), vm.ctx.new_int(fold).into())).collect();
        call_datetime_type(cls, positional, kwargs)
    })
}

unsafe extern "C" fn delta_from_delta(
    days: c_int,
    seconds: c_int,
    microseconds: c_int,
    _normalize: c_int,
    cls: *mut PyTypeObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        call_datetime_type(
            cls,
            vec![
                vm.ctx.new_int(days).into(),
                vm.ctx.new_int(seconds).into(),
                vm.ctx.new_int(microseconds).into(),
            ],
            KwArgs::default(),
        )
    })
}

unsafe extern "C" fn timezone_from_timezone(
    offset: *mut PyObject,
    name: *mut PyObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let datetime = match vm.import("datetime", 0) {
            Ok(datetime) => datetime,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let timezone = match datetime.get_attr("timezone", vm) {
            Ok(timezone) => timezone,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let mut positional = Vec::new();
        if !offset.is_null() {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(offset) });
        }
        if !name.is_null() {
            positional.push(unsafe { ptr_to_pyobject_ref_borrowed(name) });
        }
        match timezone.call_with_args(FuncArgs::new(positional, KwArgs::default()), vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn datetime_from_timestamp(
    cls: *mut PyTypeObject,
    args: *mut PyObject,
    kwargs: *mut PyObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let cls = unsafe { ptr_to_pyobject_ref_borrowed(cls.cast()) };
        let fromtimestamp = match cls.get_attr("fromtimestamp", vm) {
            Ok(method) => method,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        match fromtimestamp.call(
            FuncArgs::new(
                if args.is_null() {
                    Vec::new()
                } else {
                    match unsafe { ptr_to_pyobject_ref_borrowed(args) }
                        .try_into_value::<rustpython_vm::builtins::PyTupleRef>(vm)
                    {
                        Ok(tuple) => tuple.as_slice().to_vec(),
                        Err(_) => {
                            set_vm_exception(
                                vm.new_type_error("expected tuple args for datetime.fromtimestamp"),
                            );
                            return std::ptr::null_mut();
                        }
                    }
                },
                if kwargs.is_null() {
                    KwArgs::default()
                } else {
                    match unsafe { ptr_to_pyobject_ref_borrowed(kwargs) }
                        .try_into_value::<rustpython_vm::builtins::PyDictRef>(vm)
                    {
                        Ok(dict) => dict
                            .into_iter()
                            .filter_map(|(k, v)| {
                                k.str(vm)
                                    .ok()
                                    .map(|s| (AsRef::<str>::as_ref(&s).to_owned(), v))
                            })
                            .collect(),
                        Err(_) => {
                            set_vm_exception(
                                vm.new_type_error(
                                    "expected dict kwargs for datetime.fromtimestamp",
                                ),
                            );
                            return std::ptr::null_mut();
                        }
                    }
                },
            ),
            vm,
        ) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

unsafe extern "C" fn date_from_timestamp(
    cls: *mut PyTypeObject,
    args: *mut PyObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let cls = unsafe { ptr_to_pyobject_ref_borrowed(cls.cast()) };
        let fromtimestamp = match cls.get_attr("fromtimestamp", vm) {
            Ok(method) => method,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let positional = if args.is_null() {
            Vec::new()
        } else {
            match unsafe { ptr_to_pyobject_ref_borrowed(args) }
                .try_into_value::<rustpython_vm::builtins::PyTupleRef>(vm)
            {
                Ok(tuple) => tuple.as_slice().to_vec(),
                Err(_) => {
                    set_vm_exception(
                        vm.new_type_error("expected tuple args for date.fromtimestamp"),
                    );
                    return std::ptr::null_mut();
                }
            }
        };
        match fromtimestamp.call_with_args(FuncArgs::new(positional, KwArgs::default()), vm) {
            Ok(obj) => pyobject_ref_to_ptr(obj),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

pub unsafe fn import_datetime_api() -> *mut PyDateTime_CAPI {
    rustpython_runtime::with_vm(|vm| {
        let datetime = match import_datetime(vm) {
            Ok(datetime) => datetime,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let load_type = |name: &'static str| -> Result<
            *mut PyTypeObject,
            rustpython_vm::builtins::PyBaseExceptionRef,
        > {
            datetime
                .get_attr(name, vm)
                .map(|obj| pyobject_ref_as_ptr(&obj).cast::<PyTypeObject>())
        };

        let date_type = match load_type("date") {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let datetime_type = match load_type("datetime") {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let time_type = match load_type("time") {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let delta_type = match load_type("timedelta") {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let tzinfo_type = match load_type("tzinfo") {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        let timezone_utc = match datetime
            .get_attr("timezone", vm)
            .and_then(|timezone| timezone.get_attr("utc", vm))
        {
            Ok(obj) => pyobject_ref_as_ptr(&obj),
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };

        Box::into_raw(Box::new(PyDateTime_CAPI {
            DateType: date_type,
            DateTimeType: datetime_type,
            TimeType: time_type,
            DeltaType: delta_type,
            TZInfoType: tzinfo_type,
            TimeZone_UTC: timezone_utc,
            Date_FromDate: date_from_date,
            DateTime_FromDateAndTime: datetime_from_date_and_time,
            Time_FromTime: time_from_time,
            Delta_FromDelta: delta_from_delta,
            TimeZone_FromTimeZone: timezone_from_timezone,
            DateTime_FromTimestamp: datetime_from_timestamp,
            Date_FromTimestamp: date_from_timestamp,
            DateTime_FromDateAndTimeAndFold: datetime_from_date_and_time_and_fold,
            Time_FromTimeAndFold: time_from_time_and_fold,
        }))
    })
}
