#!/bin/bash

# Activate emsdk environment. emsdk_env.sh writes a lot to stderr so we suppress
# the output. This also prevents it from complaining when emscripten isn't yet
# installed.
source "$EMSDKDIR/emsdk_env.sh" 2> /dev/null || true
