
.PHONY: frontend go rs clean lint

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
