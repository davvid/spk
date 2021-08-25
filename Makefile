VERSION = $(shell grep Version spk.spec | cut -d ' ' -f 2)

# Create a file called "config.mak" to configure variables.
-include config.mak

default: devel

.PHONY: packages
packages:
	$(MAKE) -C packages packages

packages.%:
	$(MAKE) -C packages $*

.PHONY: clean
clean: packages.clean

.PHONY: devel
devel:
	pipenv run -- python setup.py develop

.PHONY: test
test:
	mkdir -p /tmp/spfs-runtimes
	SPFS_STORAGE_RUNTIMES="/tmp/spfs-runtimes" \
	pipenv run -- spfs run - -- pytest -x -vvv

.PHONY: rpm
rpm: SPFS_PULL_USERNAME ?= $(shell read -p "Github Username: " user; echo $$user)
rpm: SPFS_PULL_PASSWORD ?= $(shell read -s -p "Github Password/Access Token: " pass; echo $$pass)
rpm:
	cd $(SOURCE_ROOT)
	docker build . \
		-f rpmbuild.Dockerfile \
		--build-arg VERSION=$(VERSION) \
		--build-arg SPFS_PULL_USERNAME=$(SPFS_PULL_USERNAME) \
		--build-arg SPFS_PULL_PASSWORD=$(SPFS_PULL_PASSWORD) \
		--tag spk-rpm-builder
	mkdir -p dist/rpm
	CONTAINER=$$(docker create spk-rpm-builder) \
	  && docker cp $$CONTAINER:/root/rpmbuild/RPMS dist/rpm/ \
	  && docker rm --force $$CONTAINER
