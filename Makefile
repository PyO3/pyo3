.PHONY: default build test doc extensions clean

default: test extensions

build:
	cargo build

test: build
	cargo test

doc: build
	cargo doc --no-deps

extensions: build
	make -C extensions/

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

