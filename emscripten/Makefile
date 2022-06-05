CURDIR=$(abspath .)

BUILDROOT ?= $(CURDIR)/builddir
export EMSDKDIR = $(BUILDROOT)/emsdk

PLATFORM=wasm32_emscripten
SYSCONFIGDATA_NAME=_sysconfigdata__$(PLATFORM)

# BASH_ENV tells bash to run pyodide_env.sh on startup, which sets various
# environment variables. The next line instructs make to use bash to run each
# command.
export BASH_ENV := $(CURDIR)/env.sh
SHELL := /bin/bash

EMSCRIPTEN_VERSION=3.1.13

PYMAJORMINORMICRO ?= 3.11.0
PYPRERELEASE ?= b1

version_tuple := $(subst ., ,$(PYMAJORMINORMICRO:v%=%))
export PYMAJOR=$(word 1,$(version_tuple))
export PYMINOR=$(word 2,$(version_tuple))
export PYMICRO=$(word 3,$(version_tuple))
PYVERSION=$(PYMAJORMINORMICRO)$(PYPRERELEASE)
PYMAJORMINOR=$(PYMAJOR).$(PYMINOR)


PYTHONURL=https://www.python.org/ftp/python/$(PYMAJORMINORMICRO)/Python-$(PYVERSION).tgz
PYTHONTARBALL=$(BUILDROOT)/downloads/Python-$(PYVERSION).tgz
PYTHONBUILD=$(BUILDROOT)/build/Python-$(PYVERSION)

export PYTHONLIBDIR=$(BUILDROOT)/install/Python-$(PYVERSION)/lib

all: $(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a

$(BUILDROOT)/.exists: 
	mkdir -p $(BUILDROOT)
	touch $@


$(EMSDKDIR): $(CURDIR)/emscripten_patches/* $(BUILDROOT)/.exists
	git clone https://github.com/emscripten-core/emsdk.git --depth 1 --branch $(EMSCRIPTEN_VERSION) $(EMSDKDIR)
	$(EMSDKDIR)/emsdk install $(EMSCRIPTEN_VERSION)
	cd $(EMSDKDIR)/upstream/emscripten && cat $(CURDIR)/emscripten_patches/* | patch -p1
	$(EMSDKDIR)/emsdk activate $(EMSCRIPTEN_VERSION)


$(PYTHONTARBALL):
	[ -d $(BUILDROOT)/downloads ] || mkdir -p $(BUILDROOT)/downloads
	wget -q -O $@ $(PYTHONURL)

$(PYTHONBUILD)/.patched: $(PYTHONTARBALL)
	[ -d $(PYTHONBUILD) ] || ( \
		mkdir -p $(dir $(PYTHONBUILD));\
		tar -C $(dir $(PYTHONBUILD)) -xf $(PYTHONTARBALL) \
	)
	touch $@

$(PYTHONBUILD)/Makefile: $(PYTHONBUILD)/.patched $(BUILDROOT)/emsdk
	cd $(PYTHONBUILD) && \
	CONFIG_SITE=Tools/wasm/config.site-wasm32-emscripten \
  	emconfigure ./configure -C \
		--host=wasm32-unknown-emscripten \
		--build=$(shell $(PYTHONBUILD)/config.guess) \
		--with-emscripten-target=browser \
		--enable-wasm-dynamic-linking \
		--with-build-python=python3.11

$(PYTHONLIBDIR)/libpython$(PYMAJORMINOR).a : $(PYTHONBUILD)/Makefile
	cd $(PYTHONBUILD) && \
		emmake make -j3 libpython$(PYMAJORMINOR).a

	_PYTHON_SYSCONFIGDATA_NAME=$(SYSCONFIGDATA_NAME) _PYTHON_PROJECT_BASE=$(PYTHONBUILD) python3.11 -m sysconfig --generate-posix-vars
	cp `cat pybuilddir.txt`/$(SYSCONFIGDATA_NAME).py $(PYTHONBUILD)/Lib

	mkdir -p $(PYTHONLIBDIR)
	find $(PYTHONBUILD) -name '*.a' -exec cp {} $(PYTHONLIBDIR) \;
	cp -r $(PYTHONBUILD)/Lib $(PYTHONLIBDIR)/python$(PYMAJORMINOR)

clean:
	rm -rf $(BUILDROOT)
