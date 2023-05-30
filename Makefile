all: build


build: self-test.rs
	rustc self-test.rs

.PHONY: build
