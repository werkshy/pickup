
GO_FILES = $(shell find . -iname '*.go')

pickup: frontend
	make pickup-only

pickup-only: main.go $(GO_FILES)
	go build

frontend: deps
	cd frontend && yarn build

deps: frontend/yarn.lock frontend/package.json
	cd frontend && yarn install

# go vet requires the files in frontend/dist
lint: deps $(GO_FILES)
	go vet && cd frontend && yarn lint

.PHONY: frontend clean deps pickup
clean:
	rm -rf frontend/dist
	go clean
