project(
  'meson-starter', 'c', 'rust',
  version: '0.1.0',
  meson_version: '>=1.12',
  default_options: ['rust_std=2024'],
)

py = import('python').find_installation(pure: false)
pkg = import('rust').workspace(features: get_option('features')).package()

py.extension_module(
  'meson_starter',
  'src/lib.rs',
  rust_dependency_map: pkg.rust_dependency_map(),
  rust_args: pkg.rust_args(),
  dependencies: pkg.dependencies(),
  subdir: 'meson_starter',
  limited_api: get_option('features').contains('abi3') ? '3.9' : '',
  install: true,
)
py.install_sources('meson_starter/__init__.py', subdir: 'meson_starter')
