all:
	cargo build -vv --release
	cargo test --verbose

image:
	docker build -t ping-exporter .
