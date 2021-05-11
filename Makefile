all: hello


hello: hello.zig
	zig build-exe hello.zig -O ReleaseSmall --strip --single-threaded

.PHONY: all

