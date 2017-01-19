#!/usr/bin/env python
import sysconfig
import subprocess
import json
import os
import sys
import platform

if os.path.dirname(__file__):
    os.chdir(os.path.dirname(__file__))

if platform.system() == 'Windows' or platform.system().startswith('CYGWIN'):
    sys.exit(0) # test not supported on windows - ignore it

so_files = [
    sysconfig.get_config_var("LIBDIR")+"/"+sysconfig.get_config_var("LDLIBRARY"),
    sysconfig.get_config_var("LIBPL")+"/"+sysconfig.get_config_var("LDLIBRARY")
]
so_file = None
for name in so_files:
    if os.path.isfile(name):
        so_file = name
if not so_file:
    print('Could not find %r' % so_files)
    sys.exit(1)

so_symbols = set()
for line in subprocess.check_output(['readelf', '-Ws', so_file]).splitlines():
    if line:
        so_symbols.add(line.decode('utf-8').split()[-1])

assert 'PyList_Type' in so_symbols
assert 'PyList_New' in so_symbols

cfgs = []
if sys.version_info.major == 3:
    sys_lib = 'python3-sys'
    for i in range(4, sys.version_info.minor+1):
        cfgs += ['--cfg', 'Py_3_{}'.format(i)]
else:
    sys_lib = 'python27-sys'

interesting_config_flags = [
    "Py_USING_UNICODE",
    "Py_UNICODE_WIDE",
    "WITH_THREAD",
    "Py_DEBUG",
    "Py_REF_DEBUG",
    "Py_TRACE_REFS",
    "COUNT_ALLOCS"
]
for name in interesting_config_flags:
    if sysconfig.get_config_var(name):
        cfgs += ['--cfg', 'py_sys_config="{}"'.format(name)]
interesting_config_values = ['Py_UNICODE_SIZE']
for name in interesting_config_values:
    cfgs += ['--cfg', 'py_sys_config="{}_{}"'.format(name, sysconfig.get_config_var(name))]


json_output = subprocess.check_output(['rustc', '-Z', 'ast-json', '../{}/src/lib.rs'.format(sys_lib)] + cfgs)
doc = json.loads(json_output.decode('utf-8'))
foreign_symbols = set()
def visit(node, foreign):
    if isinstance(node, dict):
        node_node = node.get('node', None)
        if isinstance(node_node, dict) and node_node.get('variant') in ('Static', 'Fn') and foreign:
            foreign_symbols.add(node['ident'])
        if isinstance(node_node, dict) and node_node.get('variant') == 'ForeignMod':
            foreign = True
        for v in node.values():
            visit(v, foreign)
    elif isinstance(node, list):
        for v in node:
            visit(v, foreign)
    elif isinstance(node, (int, type(u''), bool, type(None))):
        pass
    else:
        raise Exception('Unsupported node type {}'.format(type(node)))
visit(doc, foreign=False)

assert 'PyList_Type' in foreign_symbols, "Failed getting statics from rustc -Z ast-json"
assert 'PyList_New' in foreign_symbols, "Failed getting functions from rustc -Z ast-json"

names = sorted(foreign_symbols - so_symbols)
if names:
    print('Symbols missing in {}:'.format(so_file))
    print('\n'.join(names))
    sys.exit(1)
else:
    print('Symbols in {} OK.'.format(so_file))

