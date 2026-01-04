.PHONY: run build release clean

run:
	cargo run

build:
	cargo zigbuild --release --target aarch64-unknown-linux-gnu

release: build
	./scripts/release.sh

clean:
	cargo clean
