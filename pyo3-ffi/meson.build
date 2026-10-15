project(
  'pyo3-ffi',
  'rust',
  meson_version : '>= 1.12',
  version : '0.30.0',
  license : 'MIT OR Apache-2.0'
)

MINIMUM_SUPPORTED_VERSION_MINOR = 8
STABLE_ABI_MAX_MINOR = 15


extra_args = []

# Registers PyO3's config names as reachable cfg expressions.
extra_args += [
  '--check-cfg', 'cfg(Py_LIMITED_API)',
  '--check-cfg', 'cfg(Py_GIL_DISABLED)',
  '--check-cfg', 'cfg(PyPy)',
  '--check-cfg', 'cfg(GraalPy)',
  '--check-cfg', 'cfg(RustPython)',
  '--check-cfg', 'cfg(pyo3_use_raw_dylib)',
  '--check-cfg', 'cfg(pyo3_meson_build)',
  '--check-cfg', 'cfg(py_sys_config, values("Py_DEBUG", "Py_REF_DEBUG", "Py_TRACE_REFS", "COUNT_ALLOCS"))',
]

# Allow 'Py_3_{minor}' cfgs from the minimum supported version up to
# the maximum minor version (+1 for development for the next).
foreach i : range(MINIMUM_SUPPORTED_VERSION_MINOR, STABLE_ABI_MAX_MINOR + 2)
  extra_args += ['--check-cfg', f'cfg(Py_3_@i@)']
endforeach


fs = import('fs')
rust = import('rust')
cargo_ws = rust.workspace()
pkg_obj = cargo_ws.package('pyo3-ffi', native : false)


features = pkg_obj.features()
message('Enabled features:', features)


# Deprecated cargo features.
if features.contains('extension-module')
  warning('The extension-module cargo feature is deprecated. ')
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
implementation = run_command(
  py, '-c', 'import sys; print(sys.implementation.name)',
  check : true,
).stdout().strip()
is_pypy = implementation == 'pypy'
is_graalpy = implementation == 'graalpy'
if is_pypy
  extra_args += ['--cfg', 'PyPy']
elif is_graalpy
  extra_args += ['--cfg', 'GraalPy']
elif implementation != 'cpython'
  error(f'Unsupported Python implementation: @implementation@')
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
  if is_pypy or is_graalpy
    error('abi3 builds on PyPy/GraalPy are not supported yet')
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
  elif is_graalpy
    py_lib_name = 'python-native'
  else
    py_lib_name = 'python' + abi + (py_debug ? '_d' : '')
  endif

  # raw-dylib linking: generate the `pyo3_libpython_link!` macro definition for
  # the auto-detected Python shared library. The build script (build.rs) does not
  # run under meson, so we generate the macro here. `structured_sources` below
  # places the generated file in the build directory next to
  # `src/impl_/libpython_link_bridge.rs`, so that the bridge can include it with
  # a plain relative path.
  import_name_type = host_machine.cpu_family() == 'x86' ? ', import_name_type = "undecorated"' : ''
  pyo3_libpython_link = custom_target(
    'pyo3_libpython_link',
    output: 'pyo3_libpython_link.rs',
    command: [py, '-c',
      '''import sys
name, import_name_type, path = sys.argv[1:4]
with open(path, "w") as f:
    f.write(f"""// @generated - do not edit.
macro_rules! pyo3_libpython_link {{
    ($abi:literal {{ $($body:tt)* }}) => {{
        #[link(name = "{name}", kind = "raw-dylib"{import_name_type})]
        unsafe extern $abi {{ $($body)* }}
    }};
}}
""")
''',
      py_lib_name, import_name_type, '@OUTPUT@'],
  )

  extra_args += ['--cfg', 'pyo3_use_raw_dylib', '--cfg', 'pyo3_meson_build']
endif


# Because pyo3-ffi has generated sources on Windows (the macro above), all of
# its sources are passed to `library()` as `structured_sources`. On Windows this
# mirrors the crate into the build directory, where the generated file takes
# the place of what `OUT_DIR` provides for cargo builds; on other platforms
# the crate is compiled directly from the source tree.
#
# The file list is captured at configure time; adding a new source file to the
# crate requires reconfiguring the build.
src_root = []
src_dirs = {}
foreach entry : run_command(
    py, '-c',
    '''import os, sys
base = os.path.join(sys.argv[1], "src")
for dirpath, dirnames, filenames in os.walk(base):
    dirnames.sort()
    for filename in sorted(filenames):
        if filename.endswith(".rs") and filename != "lib.rs":
            path = os.path.relpath(os.path.join(dirpath, filename), base).replace(os.sep, "/")
            print(os.path.dirname(path) or ".", "src/" + path, sep="|")
''',
    meson.current_source_dir(),
    check: true,
  ).stdout().strip().split('\n')
  if entry != ''
    parts = entry.split('|')
    if parts[0] == '.'
      src_root += [parts[1]]
    else
      src_dirs += {parts[0]: src_dirs.get(parts[0], []) + [parts[1]]}
    endif
  endif
endforeach

if is_windows
  src_dirs += {'impl_': src_dirs.get('impl_', []) + [pyo3_libpython_link]}
endif

ffi_sources = structured_sources(['src/lib.rs'] + src_root, src_dirs)


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


lib = pkg_obj.library('pyo3_ffi', ffi_sources, rust_args : extra_args)
dep = declare_dependency(
  link_with : lib,
  variables : {
    'features' : ','.join(features),
    'args' : '\0'.join(extra_args),
  },
  version : pkg_obj.version()
)
pkg_obj.override_dependency(dep)
