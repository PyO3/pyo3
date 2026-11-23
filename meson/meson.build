extra_args += cargo_ws.subproject('pyo3-ffi', pkg_obj.api(), native: false).dependency().get_variable('args').split('\0')

extra_args += ['--check-cfg', 'cfg(wip_feature_std)']
wip_no_std = run_command(
  import('python').find_installation(),
  '-c',
  'import os; print(os.environ.get("PYO3_WIP_NO_STD", "").strip().lower())',
  check: true,
).stdout().strip()
if not ['1', 'true'].contains(wip_no_std)
  extra_args += ['--cfg', 'wip_feature_std']
endif
