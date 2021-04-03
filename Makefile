
GO_FILES = $(shell find . -iname '*.go')

pickup: react
	make pickup-only

pickup-only: main.go $(GO_FILES)
	go build

react: deps
	cd react && yarn build

deps:
	cd react && yarn install

.PHONY: react clean deps pickup
clean:
	rm -rf react/dist
	go clean
