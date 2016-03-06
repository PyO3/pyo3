TARGETDIR=../target/debug
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) $(filter $(subst *,%,$2),$d))

.PHONY: all clean

SHELL=/bin/bash -o pipefail

ifndef PY
PY := $(word 2, $(subst ., ,$(shell python --version 2>&1)))
endif
ifndef NIGHTLY
ifeq ($(word 3, $(subst -, ,$(shell rustc --version 2>&1))),nightly)
NIGHTLY := 1
else
NIGHTLY := 0
endif
endif

all:

clean:
	-rm *.so
	-rm *.out
	-rm -r stamps

stamps:
	mkdir stamps

stamps/rust-cpython-$(PY): $(call rwildcard,../src,*.rs) Makefile | stamps
	-rm stamps/rust-cpython-*
	cd .. && make build PY=$(PY)
	touch $@

%.so: %.rs stamps/rust-cpython-$(PY)
	rustc $< -g -L $(TARGETDIR) -L $(TARGETDIR)/deps -o $@

%_expanded.rs: %.rs stamps/rust-cpython-$(PY)
	rustc $< -g -L $(TARGETDIR) -L $(TARGETDIR)/deps -Z unstable-options --pretty=expanded -o $@

hello.out: hello.so
	python$(PY) -c "import hello; hello.run(hello.val())" 2>&1 | tee $@

all: stamps/test-hello
stamps/test-hello: hello.out
	@grep "Rust says: Hello Python!" hello.out >/dev/null
	@grep "Rust got 42" hello.out >/dev/null

ifeq ($(NIGHTLY),1)
custom_type.out: custom_type.so
	python$(PY) -c "import custom_type; custom_type.MyType(42).a()" 2>&1 | tee $@

all: stamps/test-custom_type
stamps/test-custom_type: custom_type.out
	@grep "a() was called with self=42" custom_type.out >/dev/null
endif

custom_class.out: custom_class.so
	python$(PY) -c "import custom_class; custom_class.MyType(42).a()" 2>&1 | tee $@

all: stamps/test-custom_class custom_class_expanded.rs
stamps/test-custom_class: custom_class.out
	@grep "a() was called with self=42" custom_class.out >/dev/null

all: inheritance.so


