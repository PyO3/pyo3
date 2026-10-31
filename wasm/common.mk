# Shared logic to download Python source tarball for the wasm builds.

CURDIR=$(abspath .)

# These three are passed in from nox.
BUILDROOT ?= $(CURDIR)/builddir
PYTHON ?= python3
PYMAJORMINORMICRO ?= $(shell $(PYTHON) --version 2>&1 | awk '{print $$2}')
PYPRERELEASE ?=

# Set version variables.
version_tuple := $(subst ., ,$(PYMAJORMINORMICRO:v%=%))
PYMAJOR=$(word 1,$(version_tuple))
PYMINOR=$(word 2,$(version_tuple))
PYMICRO=$(word 3,$(version_tuple))
PYVERSION=$(PYMAJORMINORMICRO)$(PYPRERELEASE)
PYMAJORMINOR=$(PYMAJOR).$(PYMINOR)

PYTHONRELEASE=$(shell echo $(PYVERSION) | sed -E 's/(a|b|rc)[0-9]+$$//')
PYTHONURL=https://www.python.org/ftp/python/$(PYTHONRELEASE)/Python-$(PYVERSION).tgz
PYTHONTARBALL=$(BUILDROOT)/downloads/Python-$(PYVERSION).tgz
PYTHONBUILD=$(BUILDROOT)/build/Python-$(PYVERSION)

.DEFAULT_GOAL := all

$(BUILDROOT)/.exists:
	mkdir -p $(BUILDROOT)
	touch $@

$(PYTHONTARBALL): $(BUILDROOT)/.exists
	mkdir -p $(BUILDROOT)/downloads
	curl -sL $(PYTHONURL) -o $@

$(PYTHONBUILD)/.exists: $(PYTHONTARBALL)
	[ -d $(PYTHONBUILD) ] || ( \
		mkdir -p $(dir $(PYTHONBUILD));\
		tar -C $(dir $(PYTHONBUILD)) -xf $(PYTHONTARBALL) \
	)
	touch $@

.PHONY: prepare clean

# downloads the Python source and extracts ready for config
# parsing and build
prepare: $(PYTHONBUILD)/.exists

clean:
	rm -rf $(BUILDROOT)
