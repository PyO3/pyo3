#!/bin/bash


# emsdk_env.sh is fairly noisy, and suppress error message if the file doesn't
# exist yet (i.e. before building emsdk)
# shellcheck source=/dev/null
source "$EMSDKDIR/emsdk_env.sh" 2> /dev/null || true
EMCC_PATH=$(which emcc.py || echo ".")
EM_DIR=$(dirname "$EMCC_PATH")
export EM_DIR
