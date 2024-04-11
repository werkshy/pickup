
.PHONY: all frontend go rs clean lint

all: go rs

go:
	cd go && make pickup

frontend:
	cd go/frontend && yarn build

rs:
	cd rs && cargo build

lint:
	cd go && make lint
	cd rs && cargo clippy

clean:
	cd go && make clean
	cd rs && cargo clean
