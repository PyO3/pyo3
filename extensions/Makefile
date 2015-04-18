TARGETDIR=../target/debug
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) $(filter $(subst *,%,$2),$d))

.PHONY: all clean

all:

clean:
	-rm *.so
	-rm *.out
	-rm -r stamps

stamps:
	mkdir stamps

stamps/rust-cpython: $(call rwildcard,../src,*.rs) Makefile | stamps
	cd .. && cargo build
	touch $@

%.so: %.rs stamps/rust-cpython
	rustc $< -L $(TARGETDIR) -L $(TARGETDIR)/deps -o $@

hello.out: hello.so
	python -c "import hello; hello.run(hello.val())" 2>&1 | tee $@

all: stamps/test-hello
stamps/test-hello: hello.out
	@grep "Rust says: Hello Python!" hello.out >/dev/null
	@grep "Rust got 42" hello.out >/dev/null



