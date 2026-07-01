//! This module holds the [`InitConfig`] struct, see struct level docs for more detail.

use core::ffi::{c_char, c_int, CStr};
use core::fmt::Display;
use core::iter::FusedIterator;
use core::ops::{Deref, Index};
use core::ptr::{self, NonNull};

use crate::platform::prelude::*;

use pyo3_ffi::{
    PyInitConfig_AddModule, PyInitConfig_Create, PyInitConfig_Free, PyInitConfig_FreeStrList,
    PyInitConfig_GetError, PyInitConfig_GetExitCode, PyInitConfig_GetInt, PyInitConfig_GetStr,
    PyInitConfig_GetStrList, PyInitConfig_HasOption, PyInitConfig_SetInt, PyInitConfig_SetStr,
    PyInitConfig_SetStrList, PyObject,
};

/// Holds configuration that can be used to initialized the python interpreter.
///
/// [Here][1] is a list of configuration options.
///
/// [1]: https://docs.python.org/3/c-api/init_config.html#configuration-options
pub struct InitConfig(*mut crate::ffi::PyInitConfig);

impl Default for InitConfig {
    /// Creates a new initialization configuration using isolated configuration default values.
    fn default() -> Self {
        // SAFETY: no requirements
        let inner = unsafe { PyInitConfig_Create() };
        assert!(!inner.is_null());
        Self(inner)
    }
}

impl Drop for InitConfig {
    fn drop(&mut self) {
        // SAFETY: pointer was returned by PyInitConfig_Create
        unsafe { PyInitConfig_Free(self.0) };
    }
}

impl InitConfig {
    /// Initializes the python interpreter from the configuration. On success it returns the exit code
    /// requested by the interpreter, if present.
    ///
    /// # Panic
    /// Panics if the interpreter is already initialized.
    pub fn initialize(self) -> Result<Option<c_int>, InitConfigError> {
        // SAFETY: points to a valid config object
        let result = unsafe { crate::interpreter_lifecycle::initialize_from_config(self.0) }
            .expect("python interpreter is already initialized");
        match result {
            0 => Ok(None),
            -1 => {
                let mut exitcode = 0;
                // SAFETY: pointers are valid
                let result = unsafe { PyInitConfig_GetExitCode(self.0, &raw mut exitcode) };
                match result {
                    0 => Err(self.get_err()),
                    1 => Ok(Some(exitcode)),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    /// Check if the configuration has an option called `name`.
    pub fn has_option(&self, name: &CStr) -> bool {
        // SAFETY: pointers are valid
        (unsafe { PyInitConfig_HasOption(self.0, name.as_ptr()) }) == 1
    }

    /// Get an integer configuration option.
    pub fn get_int(&self, name: &CStr) -> Result<u64, InitConfigError> {
        let mut value = 1;
        self.check_error(
            // SAFETY: pointers are valid
            unsafe { PyInitConfig_GetInt(self.0, name.as_ptr(), &raw mut value) },
        )?;
        Ok(value)
    }

    /// Get a string configuration option.
    pub fn get_str(&self, name: &CStr) -> Result<Option<StringOption>, InitConfigError> {
        let mut value = ptr::null_mut();
        self.check_error(
            // SAFETY: pointers are valid
            unsafe { PyInitConfig_GetStr(self.0, name.as_ptr(), &raw mut value) },
        )?;
        Ok(NonNull::new(value).map(StringOption))
    }

    /// Get a string list configuration option.
    pub fn get_str_list(&self, name: &CStr) -> Result<StringListOption, InitConfigError> {
        let mut length = 0;
        let mut items = ptr::null_mut();
        self.check_error(
            // SAFETY: pointers are valid
            unsafe {
                PyInitConfig_GetStrList(self.0, name.as_ptr(), &raw mut length, &raw mut items)
            },
        )?;
        Ok(StringListOption {
            length,
            items: NonNull::new(items).unwrap(),
        })
    }

    /// Set an integer configuration option.
    pub fn set_int(&mut self, name: &CStr, value: u64) -> Result<(), InitConfigError> {
        self.check_error(
            // SAFETY: pointers are valid
            unsafe { PyInitConfig_SetInt(self.0, name.as_ptr(), value) },
        )
    }

    /// Set a string configuration option.
    pub fn set_str(&mut self, name: &CStr, value: &CStr) -> Result<(), InitConfigError> {
        self.check_error(
            // SAFETY: pointers are valid
            unsafe { PyInitConfig_SetStr(self.0, name.as_ptr(), value.as_ptr()) },
        )
    }

    /// Set a string list configuration option.
    pub fn set_str_list(&mut self, name: &CStr, items: &[&CStr]) -> Result<(), InitConfigError> {
        let mut raw_cstrs: Vec<_> = items.iter().map(|cs| cs.as_ptr()).collect();
        self.check_error(
            // SAFETY: pointers are valid
            unsafe {
                PyInitConfig_SetStrList(self.0, name.as_ptr(), items.len(), raw_cstrs.as_mut_ptr())
            },
        )
    }

    #[doc(hidden)]
    pub fn add_module(
        &mut self,
        name: &CStr,
        initfunc: extern "C" fn() -> *mut PyObject,
    ) -> Result<(), InitConfigError> {
        self.check_error(
            // SAFETY: pointers are valid
            unsafe { PyInitConfig_AddModule(self.0, name.as_ptr(), initfunc) },
        )
    }

    #[track_caller]
    fn check_error(&self, result: c_int) -> Result<(), InitConfigError> {
        match result {
            0 => Ok(()),
            -1 => Err(self.get_err()),
            n => unreachable!("PyInitConfig c-api function should return 0 or -1, got {n}"),
        }
    }

    #[track_caller]
    fn get_err(&self) -> InitConfigError {
        let mut err_message: *const c_char = ptr::null();
        assert_eq!(
            // SAFETY: pointers are valid
            (unsafe { PyInitConfig_GetError(self.0, &raw mut err_message) }),
            1,
            "PyInitConfig error message not set"
        );
        // SAFETY: error message is a null terminated UTF-8 string
        let err_message = unsafe { CStr::from_ptr(err_message).to_str().unwrap_unchecked() };
        InitConfigError(err_message.into())
    }
}

/// Error type when [`InitConfig`] methods fail
#[derive(Debug)]
pub struct InitConfigError(Box<str>);

impl Display for InitConfigError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl core::error::Error for InitConfigError {}

/// A string option for python configuration, returned by [`InitConfig::get_str`].
// NOTE: this cannot use a string because the memory needs to be freed with [`libc::free`]
pub struct StringOption(NonNull<c_char>);

impl Deref for StringOption {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: pointer received from python
        unsafe { CStr::from_ptr(self.0.as_ptr()) }.to_str().unwrap()
    }
}

impl Drop for StringOption {
    fn drop(&mut self) {
        // SAFETY: permitted (and required) as per cpython docs
        //         https://docs.python.org/3/c-api/init_config.html#c.PyInitConfig_GetStr
        unsafe { libc::free(self.0.as_ptr().cast()) };
    }
}

/// A list of strings for a python configuration option, returned by [`InitConfig::get_str_list`].
pub struct StringListOption {
    length: usize,
    items: NonNull<*mut c_char>,
}

impl Drop for StringListOption {
    fn drop(&mut self) {
        // SAFETY: permitted (and required) as per cpython docs
        //         https://docs.python.org/3/c-api/init_config.html#c.PyInitConfig_GetStrList
        unsafe { PyInitConfig_FreeStrList(self.length, self.items.as_ptr()) };
    }
}

impl Index<usize> for StringListOption {
    type Output = str;

    #[inline(always)]
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'a> IntoIterator for &'a StringListOption {
    type Item = &'a str;
    type IntoIter = StringListOptionIterator<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl StringListOption {
    /// Get a string from the list. Returns [`None`] if index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        if index >= self.length {
            None
        } else {
            let array_ptr = self.items.as_ptr();
            // SAFETY: PyInitConfig_GetStrList returns an array of null terminated UTF-8 strings
            //         and we just checked if the index is out of bounds.
            Some(unsafe {
                let item_ptr = *array_ptr.add(index);
                CStr::from_ptr(item_ptr).to_str().unwrap_unchecked()
            })
        }
    }

    /// Iterate over the list of strings
    #[inline]
    pub fn iter(&self) -> StringListOptionIterator<'_> {
        StringListOptionIterator {
            list: self,
            current: 0,
        }
    }
}

/// Iterator for [`StringListOption`]
pub struct StringListOptionIterator<'a> {
    list: &'a StringListOption,
    current: usize,
}

impl<'a> Iterator for StringListOptionIterator<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.list.length {
            let item = self.list.get(self.current).unwrap();
            self.current += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for StringListOptionIterator<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.list.length
    }
}

impl FusedIterator for StringListOptionIterator<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_option() {
        let config = InitConfig::default();
        assert!(config.has_option(c"allocator"));
        assert!(!config.has_option(c"non-existing"));
    }

    #[test]
    fn int_option() {
        let mut config = InitConfig::default();

        config.set_int(c"non-existing", 0).unwrap_err();

        config.set_int(c"optimization_level", 100).unwrap();
        assert_eq!(100, config.get_int(c"optimization_level").unwrap());

        // `base_exec_prefix` is not an int option
        config.set_int(c"base_exec_prefix", 1).unwrap_err();
    }

    #[test]
    fn str_option() {
        let mut config = InitConfig::default();

        config.set_str(c"non-existing", c"hello").unwrap_err();

        config.set_str(c"base_exec_prefix", c"/some/path").unwrap();
        assert_eq!(
            "/some/path",
            &*config.get_str(c"base_exec_prefix").unwrap().unwrap()
        );

        // `optimization_level` is not a str option
        config.set_str(c"optimization_level", c"hello").unwrap_err();
    }

    #[test]
    fn str_list_option() {
        let mut config = InitConfig::default();

        config
            .set_str_list(c"non-existing", &[c"hello"])
            .unwrap_err();

        config.set_str_list(c"argv", &[c"hello", c"world"]).unwrap();
        let list = config.get_str_list(c"argv").unwrap();
        assert_eq!("hello", list.get(0).unwrap());
        assert_eq!("world", list.get(1).unwrap());
        assert!(list.get(2).is_none());

        // `optimization_level` is not a str list option
        config
            .set_str_list(c"optimization_level", &[c"hello"])
            .unwrap_err();
    }
}
