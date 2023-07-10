all: build

build: src
	docker build -t self-test-gpu .
	gvmkit-build self-test-gpu:latest -o self-test-gpu.gvmi

.PHONY: all

.PHONY: clean
clean:
	cargo clean
	rm -f self-test-gpu.gvmi
