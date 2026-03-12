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
PYTHONTARBALL=$(BUILDROOT)/downloads/Python-$(PYVERSION).tgz
PYTHONBUILD=$(BUILDROOT)/build/Python-$(PYVERSION)

PYTHONLIBDIR=$(BUILDROOT)/install/Python-$(PYVERSION)/lib

all: $(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a

$(BUILDROOT)/.exists:
	mkdir -p $(BUILDROOT)
	touch $@



$(PYTHONTARBALL):
	[ -d $(BUILDROOT)/downloads ] || mkdir -p $(BUILDROOT)/downloads
	wget -q -O $@ $(PYTHONURL)

$(PYTHONBUILD)/.patched: $(PYTHONTARBALL)
	[ -d $(PYTHONBUILD) ] || ( \
		mkdir -p $(dir $(PYTHONBUILD));\
		tar -C $(dir $(PYTHONBUILD)) -xf $(PYTHONTARBALL) \
	)
	touch $@

# Install emscripten
$(EMSDKDIR): $(BUILDROOT)/.exists $(PYTHONBUILD)/.patched
	# NB: future Python versions will stash emscripten version in config.toml
	# EMSCRIPTEN_VERSION=$$(python -c "import tomllib; print(tomllib.load(open('$(PYTHONBUILD)/Tools/wasm/emscripten/config.toml', 'rb'))['emscripten-version'])")
	EMSCRIPTEN_VERSION=4.0.12 && \
	git clone https://github.com/emscripten-core/emsdk.git --depth 1 --branch $$EMSCRIPTEN_VERSION $(EMSDKDIR) && \
	$(EMSDKDIR)/emsdk install $$EMSCRIPTEN_VERSION && \
	$(EMSDKDIR)/emsdk activate $$EMSCRIPTEN_VERSION

$(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a: $(PYTHONBUILD)/.patched $(EMSDKDIR)
	cd $(PYTHONBUILD) && \
	source $(EMSDKDIR)/emsdk_env.sh && \
	python$(PYMAJORMINOR) Tools/wasm/emscripten build --quiet -- --config-cache --with-pydebug

	# Generate sysconfigdata
	_PYTHON_SYSCONFIGDATA_NAME=$(SYSCONFIGDATA_NAME) _PYTHON_PROJECT_BASE=$(PYTHONBUILD) python$(PYMAJORMINOR) -m sysconfig --generate-posix-vars
	cp `cat pybuilddir.txt`/$(SYSCONFIGDATA_NAME).py $(PYTHONBUILD)/Lib

	mkdir -p $(PYTHONLIBDIR)
	# Copy libexpat.a, libmpdec.a, and libpython3.11.a
	# In noxfile, we explicitly link libexpat and libmpdec via RUSTFLAGS
	find $(PYTHONBUILD) -name '*.a' -exec cp {} $(PYTHONLIBDIR) \;
	# Install Python stdlib
	cp -r $(PYTHONBUILD)/Lib $(PYTHONLIBDIR)/python$(PYMAJORMINOR)

clean:
	rm -rf $(BUILDROOT)
