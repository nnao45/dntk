GO15VENDOREXPERIMENT=1

NAME	 := dntk
TARGET	 := bin/$(NAME)
VERSION  := beta
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
	rm -rf vendor/*
	rm -rf dist/*

upde:
	dep ensure -update

dep:
	dep ensure

dep-install:
	go get github.com/golang/dep/cmd/dep
