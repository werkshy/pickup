
GO_FILES = $(shell find . -iname '*.go')

build: pickup

pickup: main.go $(GO_FILES) react
	go build

react: deps
	cd react && yarn build

deps:
	cd react && yarn install

.PHONY: react clean deps
clean:
	rm -rf react/dist
	go clean
