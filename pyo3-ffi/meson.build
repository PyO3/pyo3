project(
  'pyo3-ffi',
  'rust',
  meson_version : '>= 1.12',
  version : '0.29.2',
  license : 'MIT OR Apache-2.0'
)

MINIMUM_SUPPORTED_VERSION_MINOR = 8
STABLE_ABI_MAX_MINOR = 15
MINIMUM_SUPPORTED_VERSION_PYPY_MINOR = 11
MAXIMUM_SUPPORTED_VERSION_PYPY_MINOR = 11

extra_args = []
extra_deps = []

# Registers PyO3's config names as reachable cfg expressions.
extra_args += [
  '--check-cfg', 'cfg(Py_LIMITED_API)',
  '--check-cfg', 'cfg(Py_GIL_DISABLED)',
  '--check-cfg', 'cfg(PyPy)',
  '--check-cfg', 'cfg(GraalPy)',
  '--check-cfg', 'cfg(RustPython)',
  '--check-cfg', 'cfg(py_sys_config, values("Py_DEBUG", "Py_REF_DEBUG", "Py_TRACE_REFS", "COUNT_ALLOCS"))',
]

# Allow 'Py_3_{minor}' cfgs from the minimum supported version up to
# the maximum minor version (+1 for development for the next).
foreach i : range(MINIMUM_SUPPORTED_VERSION_MINOR, STABLE_ABI_MAX_MINOR + 2)
  extra_args += ['--check-cfg', f'cfg(Py_3_@i@)']
endforeach

# Allow all possible 'pyo3_dll' cfg values for raw-dylib linking on Windows.
dll_names = ['python3', 'python3_d', 'python3t', 'python3t_d']

foreach i : range(MINIMUM_SUPPORTED_VERSION_MINOR, STABLE_ABI_MAX_MINOR + 2)
  # CPython DLL names.
  dll_names += [f'python3@i@', f'python3@i@_d']
  if i >= 13
    dll_names += [f'python3@i@t', f'python3@i@t_d']
  endif
endforeach

foreach i : range(MINIMUM_SUPPORTED_VERSION_PYPY_MINOR, MAXIMUM_SUPPORTED_VERSION_PYPY_MINOR + 1)
  # PyPy DLL names.
  dll_names += [f'libpypy3.@i@-c']
endforeach

dll_names_quoted = []
foreach name : dll_names
  dll_names_quoted += f'"@name@"'
endforeach

pyo3_dll_values = ', '.join(dll_names_quoted)
extra_args += ['--check-cfg', f'cfg(pyo3_dll, values(@pyo3_dll_values@))']


fs = import('fs')
rust = import('rust')
cargo_ws = rust.workspace()
pkg_obj = cargo_ws.package('pyo3-ffi', native : false)


features = pkg_obj.features()
message('Enabled features:', features)


# Deprecated cargo features.
if features.contains('extension-module')
  warning('The extension-module cargo feature is deprecated. Use the "extension_module" Meson option instead.')
endif
if features.contains('generate-import-lib')
  warning('The generate-import-lib cargo feature is deprecated')
endif


py = import('python').find_installation()
py_minor_version = py.language_version().split('.')[1].to_int()

is_windows = host_machine.system() == 'windows'
py_gil_disabled = py.get_variable('Py_GIL_DISABLED', 0) == 1
py_debug = py.get_variable('Py_DEBUG', 0) == 1


# Implementation detection.
soabi = py.get_variable('SOABI', 'cpython')
is_pypy = soabi.startswith('pypy')
if is_pypy
  extra_args += ['--cfg', 'PyPy']
endif


# Python ABI.
if features.contains('abi3t')
  if not py_gil_disabled
    error('abi3t required but Python is not free-threaded')
  endif
  abi = '3t'
  limited_api = true
elif features.contains('abi3')
  if py_gil_disabled
    error('abi3 required but Python is free-threaded')
  endif
  abi = '3'
  limited_api = true
else
  abi = f'3@py_minor_version@' + (py_gil_disabled ? 't' : '')
  limited_api = false
endif

if py_gil_disabled
  extra_args += ['--cfg', 'Py_GIL_DISABLED']
endif


# Emit cumulative 'Py_3_{minor}' cfgs up to the interpreter version,
# or up to the abi3 floor when targeting the stable ABI.
py_minor_version_max = py_minor_version
if limited_api
  if is_pypy
    error('abi3 required but PyPy does not support it')
  endif
  if is_windows and py_debug
    error('no stable-ABI python3.dll exists for debug CPython on Windows')
  endif

  extra_args += ['--cfg', 'Py_LIMITED_API']
  # Parse feature in the form 'abi3-py3{minor}' or 'abi3t-py3{minor}'
  # to extract the minor version.
  foreach f : features
    if f.startswith('abi3-py3')
      floor = f.substring(8).to_int()
    elif f.startswith('abi3t-py3')
      floor = f.substring(9).to_int()
    else
      continue
    endif
    py_minor_version_max = floor < py_minor_version_max ? floor : py_minor_version_max
  endforeach
endif

foreach m : range(MINIMUM_SUPPORTED_VERSION_MINOR, py_minor_version_max + 1)
  extra_args += ['--cfg', f'Py_3_@m@']
endforeach


foreach flag : ['Py_DEBUG', 'Py_REF_DEBUG', 'Py_TRACE_REFS', 'COUNT_ALLOCS']
  if py.get_variable(flag, 0) == 1
    extra_args += ['--cfg', f'py_sys_config="@flag@"']
  endif
endforeach


if is_windows
  # Windows: extension modules need to link to libpython.
  if is_pypy
    py_lib_name = f'libpypy3.@py_minor_version@-c'
  else
    py_lib_name = 'python' + abi + (py_debug ? '_d' : '')
  endif
  if get_option('use_raw_dylib')
    extra_args += ['--cfg', 'pyo3_use_raw_dylib', '--cfg', f'pyo3_dll="@py_lib_name@"']
  else
    error('configuration currently not supported')
    # extra_args += ['-l', f'pythonXY:@py_lib_name@']
    # extra_deps += py.dependency()
  endif
else
  # Unix: extension modules never link libpython, only embedding links it.
  if not get_option('extension_module')
    extra_deps += py.dependency(embed : true)
  endif
endif


# Compiler features used by PyO3.
rustc_features = {
  'const_is_null': 84,
  'fn_ptr_eq': 85,
  'from_bytes_with_nul_error': 86,
  'cfg_select': 95,
}
rustc_minor_version = meson.get_compiler('rust').version().split('.')[1].to_int()
foreach cfg, minor_version_required : rustc_features
  extra_args += ['--check-cfg', f'cfg(@cfg@)']
  if rustc_minor_version >= minor_version_required
    extra_args += ['--cfg', cfg]
  endif
endforeach


lib = pkg_obj.library('pyo3_ffi', dependencies : extra_deps, rust_args : extra_args)
dep = declare_dependency(
  link_with : lib,
  variables : {
    'features' : ','.join(features),
    'args' : '\0'.join(extra_args),
    'extension_module' : get_option('extension_module').to_string(),
  },
  version : pkg_obj.version()
)
pkg_obj.override_dependency(dep)
