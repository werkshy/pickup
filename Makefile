
GO_FILES = $(shell find . -iname '*.go')

build: pickup react

pickup: main.go $(GO_FILES)
	go build

react:
	cd react && yarn build

deps:
	cd react && yarn install

.PHONY: react clean deps
clean:
	rm -rf react/dist
	go clean
