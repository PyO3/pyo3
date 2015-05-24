.PHONY: default build test doc extensions clean

ifndef PY
PY=3
endif

ifeq ($(PY),2)
FEATURES=--features python27-sys --no-default-features
endif
ifeq ($(PY),3)
FEATURES=--features python3-sys --no-default-features
endif

default: test extensions

build:
	cargo build $(FEATURES)

test: build
	cargo test $(FEATURES)

doc: build
	cargo doc --no-deps $(FEATURES)

extensions: build
	make -C extensions/ PY=$(PY)

clean:
	rm -r target
	make -C extensions/ clean

gh-pages:
	git clone --branch gh-pages git@github.com:dgrunwald/rust-cpython.git gh-pages

.PHONY: gh-pages-doc
gh-pages-doc: doc | gh-pages
	cd gh-pages && git pull
	rm -r gh-pages/doc
	cp -r target/doc gh-pages/
	rm gh-pages/doc/.lock
	cd gh-pages && git add .
	cd gh-pages && git commit -m "Update documentation"

publish: default gh-pages-doc
	cargo publish
	cd gh-pages && git push

