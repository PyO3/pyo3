#!/usr/local/bin/python
import pathlib
import sys
import subprocess

p = pathlib.Path(sys.argv[1])

sys.exit(subprocess.call(["node", p.name], cwd=p.parent))
