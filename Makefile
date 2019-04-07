
GO_FILES = $(shell find . -iname '*.go')

build: pickup react

pickup: main.go $(GO_FILES)
	go build

react:
	cd react && npm run build

.PHONY: react clean
clean:
	rm -rf react/dist
	go clean
