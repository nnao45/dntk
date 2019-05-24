# general
VERSION = $(shell ./version.sh)
NAME = twiquery-stream
TARGET = $(NAME)
DOCKER_REPO = nnao45
LINUX_TERM_LIB = linux_unknown.rs

$(TARGET):
	cargo build --release

.PHONY: version
version:
	@echo $(VERSION)

.PHONY: run
run:
	cargo run --release

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	cargo clean

.PHONY: docker-build
docker-build:
	docker rmi -f $(DOCKER_REPO)/$(TARGET):latest
	docker build -t $(DOCKER_REPO)/$(TARGET):latest .
	docker tag $(DOCKER_REPO)/$(TARGET):latest $(DOCKER_REPO)/$(TARGET):$(VERSION)

.PHONY: docker-push
docker-push:
	docker push $(DOCKER_REPO)/$(NAME):latest
	docker push $(DOCKER_REPO)/$(NAME):$(VERSION)

.PHONY: docker-release
docker-release: docker-build docker-push