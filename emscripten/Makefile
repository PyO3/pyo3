CURDIR=$(abspath .)

# These three are passed in from nox.
BUILDROOT ?= $(CURDIR)/builddir
PYMAJORMINORMICRO ?= $(shell python --version 2>&1 | awk '{print $$2}')

export EMSDKDIR = $(BUILDROOT)/emsdk

PLATFORM=wasm32_emscripten
SYSCONFIGDATA_NAME=_sysconfigdata__$(PLATFORM)

# BASH_ENV tells bash to source emsdk_env.sh on startup.
export BASH_ENV := $(CURDIR)/env.sh
# Use bash to run each command so that env.sh will be used.
SHELL := /bin/bash


# Set version variables.
version_tuple := $(subst ., ,$(PYMAJORMINORMICRO:v%=%))
PYMAJOR=$(word 1,$(version_tuple))
PYMINOR=$(word 2,$(version_tuple))
PYMICRO=$(word 3,$(version_tuple))
PYVERSION=$(PYMAJORMINORMICRO)
PYMAJORMINOR=$(PYMAJOR).$(PYMINOR)


PYTHONURL=https://www.python.org/ftp/python/$(PYMAJORMINORMICRO)/Python-$(PYVERSION).tgz
# TODO: resume download once 3.14.4 ships with emscripten cache
# PYTHONTARBALL=$(BUILDROOT)/downloads/Python-$(PYVERSION).tgz
# PYTHONBUILD=$(BUILDROOT)/build/Python-$(PYVERSION)
PYTHONBUILD=$(BUILDROOT)/build/cpython

PYTHONLIBDIR=$(BUILDROOT)/install/Python-$(PYVERSION)/lib

CROSS_PYTHON=$(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python/python.sh

all: $(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a

$(BUILDROOT)/.exists:
	mkdir -p $(BUILDROOT)
	touch $@

# TODO: use tarball once 3.14.4 ships with emscripten cache

# $(PYTHONTARBALL):
# 	[ -d $(BUILDROOT)/downloads ] || mkdir -p $(BUILDROOT)/downloads
# 	wget -q -O $@ $(PYTHONURL)

# $(PYTHONBUILD)/.patched: $(PYTHONTARBALL)
# 	[ -d $(PYTHONBUILD) ] || ( \
# 		mkdir -p $(dir $(PYTHONBUILD));\
# 		tar -C $(dir $(PYTHONBUILD)) -xf $(PYTHONTARBALL) \
# 	)
# 	touch $@

ifneq ($(PYMAJORMINOR),3.14)
$(error PYMAJORMINOR must be 3.14, got '$(PYMAJORMINOR)')
endif

$(PYTHONBUILD)/.patched: $(BUILDROOT)/.exists
	[ -d $(PYTHONBUILD) ] || ( \
		mkdir -p $(dir $(PYTHONBUILD));\
		git clone --depth 1 --branch 3.14 https://github.com/python/cpython $(PYTHONBUILD) \
	)
	touch $@

$(CROSS_PYTHON): $(PYTHONBUILD)/.patched
	cd $(PYTHONBUILD) && \
	python$(PYMAJORMINOR) Tools/wasm/emscripten install-emscripten --quiet --emsdk-cache=./emsdk-cache && \
	python$(PYMAJORMINOR) Tools/wasm/emscripten build --quiet --emsdk-cache=./emsdk-cache

$(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a: $(CROSS_PYTHON)
	# Generate sysconfigdata
	_PYTHON_SYSCONFIGDATA_NAME=$(SYSCONFIGDATA_NAME) _PYTHON_PROJECT_BASE=$(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python $(CROSS_PYTHON) -m sysconfig --generate-posix-vars
	cp `cat pybuilddir.txt`/$(SYSCONFIGDATA_NAME).py $(PYTHONBUILD)/Lib

	mkdir -p $(PYTHONLIBDIR)
	# Make a static library for _hacl, for some reason these are missing from the build?
	cd $(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python && \
		source $(EMSDKDIR)/emsdk_env.sh && \
		emar rcs Modules/_hacl/libhacl.a Modules/_hacl/*.o
	# Copy all .a libraries
	find $(PYTHONBUILD)/cross-build/wasm32-emscripten/ -name '*.a' -exec cp {} $(PYTHONLIBDIR) \;
	# Install Python stdlib
	cp -r $(PYTHONBUILD)/Lib $(PYTHONLIBDIR)/python$(PYMAJORMINOR)

clean:
	rm -rf $(BUILDROOT)
