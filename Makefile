GO15VENDOREXPERIMENT = 1
OSXCROSS_NO_INCLUDE_PATH_WARNINGS = 1
VERSION = v1.0.11

NAME	 := dntk
TARGET	 := bin/$(NAME)
PRE-VERSION := $(shell grep 'Current' README.md | tr -d '***' | rev |cut -c 1-6 | rev)
DIST_DIRS := find * -type d -exec
SRCS	:= $(shell find . -type f -name '*.go')
LDFLAGS := -ldflags="-s -X \"main.version=$(VERSION)\""
OPTS :=-a -installsuffix cgo

$(TARGET): $(SRCS)
	go build $(OPTS) $(LDFLAGS) -o bin/$(NAME) pkg/dntk.go 

.PHONY: install
install:
	go install $(LDFLAGS)

.PHONY: clean
clean:
	rm -rf bin/*

.PHONY: clean-all
clean-all:
	rm -rf bin/*
	rm -rf dist/*

.PHONY: run
run:
	go run $(NAME).go

.PHONY: upde
upde:
	dep ensure -update

.PHONY: deps
dep:
	dep ensure

.PHONY: dep-install
dep-install:
	go get -u github.com/golang/dep/cmd/dep

.PHONY: readme-upde
readme-upde:
	sed -i '' -e 's/$(PRE-VERSION)/$(VERSION)/g' README.md
	sed -i '' -e 's/$(PRE-VERSION)/$(VERSION)/g' Dockerfile

.PHONY: release
release:
	git tag -a $(VERSION) -m 'version $(VERSION)' ; git push origin $(VERSION)

.PHONY: cross-build
cross-build: deps
	GOOS=darwin GOARCH=amd64 CGO_ENABLED=1 go build $(OPTS) $(LDFLAGS) -o dist/$(NAME)-darwin-amd64/$(NAME) pkg/dntk.go
	GOOS=darwin GOARCH=386 CGO_ENABLED=1 go build $(OPTS) $(LDFLAGS) -o dist/$(NAME)-darwin-386/$(NAME) pkg/dntk.go
	CC=x86_64-pc-linux-gcc GOOS=linux GOARCH=amd64 CGO_ENABLED=1 go build $(OPTS) $(LDFLAGS) -o dist/$(NAME)-linux-amd64/$(NAME) pkg/dntk.go
	CC=i586-pc-linux-gcc GOOS=linux GOARCH=386 CGO_ENABLED=1 go build $(OPTS) $(LDFLAGS) -o dist/$(NAME)-linux-386/$(NAME) pkg/dntk.go

.PHONY: dist
dist:
	cd dist && \
		$(DIST_DIRS) cp ../LICENSE {} \; && \
		$(DIST_DIRS) cp ../README.md {} \; && \
		$(DIST_DIRS) tar -zcf {}-$(VERSION).tar.gz {} \; && \
		$(DIST_DIRS) zip -r {}-$(VERSION).zip {} \; && \
		cd ..

.PHONY: deploy
deploy: clean readme-upde release clean-all