GO15VENDOREXPERIMENT=1

NAME	 := dntk
TARGET	 := bin/$(NAME)
VERSION  := v1.0.8
PRE-VERSION := $(shell grep 'Current' README.md | tr -d '***' | rev |cut -c 1-6 | rev)
DIST_DIRS := find * -type d -exec

SRCS	:= $(shell find . -type f -name '*.go')
LDFLAGS := -ldflags="-s -w -X \"main.version=$(VERSION)\" -extldflags \"-static\""

$(TARGET): $(SRCS)
	go build -a -tags netgo -installsuffix netgo $(LDFLAGS) -o bin/$(NAME)

.PHONY: install
install:
	go install $(LDFLAGS)

.PHONY: clean
clean:
	rm -rf bin/*

.PHONY: clean-all
clean-all:
	rm -rf bin/*
	rm -rf vendor/*
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
	go get github.com/golang/dep/cmd/dep

.PHONY: readme-upde
readme-upde:
	sed -i -e 's/$(PRE-VERSION)/$(VERSION)/g' README.md
	rm -f README.md-e 

.PHONY: release
release:
	git tag -a $(VERSION) -m 'version $(VERSION)' ; git push --tags origin master

.PHONY: cross-build
cross-build: deps
	for os in darwin linux; do \
		for arch in amd64 386; do \
			GOOS=$$os GOARCH=$$arch CGO_ENABLED=0 go build -a -tags netgo -installsuffix netgo $(LDFLAGS) -o dist/$(NAME)-$$os-$$arch/$(NAME); \
		done; \
	done

.PHONY: dist
dist:
	cd dist && \
		$(DIST_DIRS) cp ../LICENSE {} \; && \
		$(DIST_DIRS) cp ../README.md {} \; && \
		$(DIST_DIRS) tar -zcf {}-$(VERSION).tar.gz {} \; && \
		$(DIST_DIRS) zip -r {}-$(VERSION).zip {} \; && \
		cd ..