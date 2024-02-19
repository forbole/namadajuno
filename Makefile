#!/bin/sh

NAMADA_VERSION := 0.31.1
BASE_URL := https://raw.githubusercontent.com/anoma/namada
URL := $(BASE_URL)/v$(NAMADA_VERSION)/wasm/checksums.json

CHECK_CURL := $(shell command -v curl 2> /dev/null)
CHECK_WGET := $(shell command -v wget 2> /dev/null)

ifdef CHECK_CURL
DOWNLOAD_CMD = curl -o
else ifdef CHECK_WGET
DOWNLOAD_CMD = wget -O
else
$(error Neither curl nor wget are available on your system)
endif

# Determine the OS and set the appropriate value for OS
UNAME := $(shell uname)
ifeq ($(UNAME),Linux)
    OS := linux
endif
ifeq ($(UNAME),Darwin)
    OS := osx
endif

# Set a default value for OS if it's not recognized
OS ?= linux

download-checksum:
	@if [ ! -f checksums.json ]; then \
		echo $(URL); \
		$(DOWNLOAD_CMD) checksums.json $(URL); \
	fi


stop-docker-test:
	@echo "Stopping Docker container..."
	@docker stop bdjuno-test-db || true && docker rm bdjuno-test-db || true
.PHONY: stop-docker-test

start-docker-test: stop-docker-test
	@echo "Starting Docker container..."
	@docker run --name bdjuno-test-db -e POSTGRES_USER=bdjuno -e POSTGRES_PASSWORD=password -e POSTGRES_DB=bdjuno -d -p 6433:5432 postgres
.PHONY: start-docker-test