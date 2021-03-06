# general
VERSION = $(shell ./version.sh)
PRE-VERSION := $(shell grep 'Current' README.md | tr -d '***' | rev |cut -c 1-7 | rev)
NAME = dntk
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


.PHONY: docker-login
docker-login:
	echo ${DOCKERHUB_PASSWORD} | docker login -u nnao45 --password-stdin

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

.PHONY: git-release
git-release:
	git add .
	git commit -m "release $(VERSION)"
	git tag -a $(VERSION) -m "release $(VERSION)"
	git push origin $(VERSION)

.PHONY: cargo-release
cargo-release:
	cargo publish

.PHONY: readme-upde
readme-upde:
	sed -i '' -e 's/$(PRE-VERSION)/$(VERSION)/g' README.md

.PHONY: toml-upde
toml-upde:
	@./release.sh

.PHONY: release
release: toml-upde readme-upde git-release cargo-release docker-release