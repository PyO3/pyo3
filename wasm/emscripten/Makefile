include ../common.mk

NODE_VERSION=24.18.0

PLATFORM=wasm32_emscripten
SYSCONFIGDATA_NAME=_sysconfigdata__$(PLATFORM)

export EMSDKDIR = $(PYTHONBUILD)/emsdk-cache

PYTHONLIBDIR=$(BUILDROOT)/install/Python-$(PYVERSION)/lib
CROSS_PYTHON=$(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python/python.sh

# Download Node.js when building locally; in CI it is provided by actions/setup-node.
ifdef GITHUB_ACTIONS
NODE_DEP=
NODE_BIN=$(shell which node)
else
NODE_DIR=$(BUILDROOT)/node
NODE_DEP=$(NODE_DIR)/.exists
NODE_BIN=$(NODE_DIR)/bin/node
NODE_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
NODE_ARCH := $(shell uname -m | sed -e 's/x86_64/x64/' -e 's/aarch64/arm64/')
NODE_URL=https://nodejs.org/dist/v$(NODE_VERSION)/node-v$(NODE_VERSION)-$(NODE_OS)-$(NODE_ARCH).tar.xz
endif

# Prepend the downloaded Node.js to PATH (no-op in CI, where setup-node provides it).
export PATH := $(NODE_BIN)$(PATH)

all: $(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a

$(NODE_DIR)/.exists: $(BUILDROOT)/.exists
	[ -d $(NODE_DIR) ] || mkdir -p $(NODE_DIR)
	curl -s -S --location $(NODE_URL) | \
		tar --strip-components 1 --directory $(NODE_DIR) --extract --xz
	touch $@

$(CROSS_PYTHON): $(PYTHONBUILD)/.exists $(NODE_DEP)
	cd $(PYTHONBUILD) && \
	$(PYTHON) Platforms/emscripten install-emscripten --quiet --emsdk-cache=$(EMSDKDIR) && \
	$(PYTHON) Platforms/emscripten build --quiet --emsdk-cache=$(EMSDKDIR) --host-runner=$(NODE_BIN)

$(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a: $(CROSS_PYTHON)
	# Generate sysconfigdata
	_PYTHON_SYSCONFIGDATA_NAME=$(SYSCONFIGDATA_NAME) _PYTHON_PROJECT_BASE=$(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python $(CROSS_PYTHON) -m sysconfig --generate-posix-vars
	cp `cat pybuilddir.txt`/$(SYSCONFIGDATA_NAME).py $(PYTHONBUILD)/Lib

	mkdir -p $(PYTHONLIBDIR)
	# Make a static library for _hacl, for some reason these are missing from the build?
	# source emsdk_env.sh to get emar in PATH, works best when done from the emsdk directory
	EMSDK_ENV=$$(find $(PYTHONBUILD)/emsdk-cache -name 'emsdk_env.sh' | head -n 1) && \
	    cd $$(dirname $$EMSDK_ENV) && \
		. $$EMSDK_ENV && \
	    cd $(PYTHONBUILD)/cross-build/wasm32-emscripten/build/python && \
		emar rcs Modules/_hacl/libhacl.a Modules/_hacl/*.o
	# Copy all .a libraries
	find $(PYTHONBUILD)/cross-build/wasm32-emscripten/ -name '*.a' -exec cp {} $(PYTHONLIBDIR) \;
	# Install Python stdlib
	cp -r $(PYTHONBUILD)/Lib $(PYTHONLIBDIR)/python$(PYMAJORMINOR)
