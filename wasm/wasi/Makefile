include ../common.mk

WASI_SDK_VERSION=24
WASMTIME_VERSION=46.0.1

CONFIG_SITE=$(PYTHONBUILD)/Tools/wasm/wasi/config.site-wasm32-wasi
CROSSBUILD=$(PYTHONBUILD)/cross-build/wasm32-wasip1
LIBDIR=$(CROSSBUILD)/build/lib.wasi-wasm32-$(PYMAJORMINOR)

# In CI the WASI SDK is installed by bytecodealliance/setup-wasi-sdk-action, which sets
# WASI_SDK_PATH; otherwise download the release asset matching this host into the build dir.
ifdef GITHUB_ACTIONS
WASI_SDK_DIR=$(WASI_SDK_PATH)
WASI_SDK_DEP=
else
WASI_SDK_DIR=$(BUILDROOT)/wasi-sdk
WASI_SDK_DEP=$(WASI_SDK_DIR)/.exists
WASI_SDK_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]' | sed 's/darwin/macos/')
WASI_SDK_ARCH := $(shell uname -m | sed 's/aarch64/arm64/')
WASI_SDK_URL=https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$(WASI_SDK_VERSION)/wasi-sdk-$(WASI_SDK_VERSION).0-$(WASI_SDK_ARCH)-$(WASI_SDK_OS).tar.gz
endif

# Similar for wasmtime, which is otherwise expected on PATH.
ifdef GITHUB_ACTIONS
WASMTIME_DEP=
WASMTIME_PATH=
else
WASMTIME_DIR=$(BUILDROOT)/wasmtime
WASMTIME_DEP=$(WASMTIME_DIR)/.exists
WASMTIME_PATH=$(WASMTIME_DIR):
WASMTIME_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]' | sed 's/darwin/macos/')
# NB wasmtime names artifacts with aarch64 architecture, unlike wasi which uses arm64
WASMTIME_ARCH := $(shell uname -m | sed 's/arm64/aarch64/')
WASMTIME_URL=https://github.com/bytecodealliance/wasmtime/releases/download/v$(WASMTIME_VERSION)/wasmtime-v$(WASMTIME_VERSION)-$(WASMTIME_ARCH)-$(WASMTIME_OS).tar.xz
endif

all: $(LIBDIR)/libpython$(PYMAJORMINOR).a

$(WASI_SDK_DIR)/.exists: $(BUILDROOT)/.exists
	[ -d $(WASI_SDK_DIR) ] || mkdir -p $(WASI_SDK_DIR)
	curl -s -S --location $(WASI_SDK_URL) | \
		tar --strip-components 1 --directory $(WASI_SDK_DIR) --extract --gunzip
	touch $@

$(WASMTIME_DIR)/.exists: $(BUILDROOT)/.exists
	[ -d $(WASMTIME_DIR) ] || mkdir -p $(WASMTIME_DIR)
	curl -s -S --location $(WASMTIME_URL) | \
		tar --strip-components 1 --directory $(WASMTIME_DIR) --extract --xz
	touch $@

$(PYTHONBUILD)/.patched: $(PYTHONBUILD)/.exists
	# Force-disable POSIX dynamic loading for WASI
	grep -q '^ac_cv_func_dlopen=no' $(CONFIG_SITE) || \
		printf '\nac_cv_func_dlopen=no\nac_cv_lib_dl_dlopen=no\n' >> $(CONFIG_SITE)
	touch $@

$(LIBDIR)/libpython$(PYMAJORMINOR).a: $(PYTHONBUILD)/.patched $(WASI_SDK_DEP) $(WASMTIME_DEP)
	cd $(PYTHONBUILD) && \
		WASI_SDK_PATH=$(WASI_SDK_DIR) \
		PATH=$(WASMTIME_PATH)$(PATH) \
		$(PYTHON) Tools/wasm/wasi build --quiet -- --config-cache
	# Collect the static libraries the test build links against
	cp $(CROSSBUILD)/libpython$(PYMAJORMINOR).a \
	   $(CROSSBUILD)/Modules/_hacl/libHacl_HMAC.a \
	   $(CROSSBUILD)/Modules/_decimal/libmpdec/libmpdec.a \
	   $(CROSSBUILD)/Modules/expat/libexpat.a \
	   $(LIBDIR)
