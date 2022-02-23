
GO_FILES = $(shell find . -iname '*.go')

pickup: react
	make pickup-only

pickup-only: main.go $(GO_FILES)
	go build

react: deps
	cd react && yarn build

deps: react/yarn.lock react/package.json
	cd react && yarn install

# go vet requires the files in react/dist
lint: deps $(GO_FILES)
	go vet && cd react && yarn lint

.PHONY: react clean deps pickup
clean:
	rm -rf react/dist
	go clean
