#!make
SHELL:=/bin/bash

# pp - pretty print function
yellow := $(shell tput setaf 3)
normal := $(shell tput sgr0)
define pp
	@printf '$(yellow)$(1)$(normal)\n'
endef


help: Makefile
	@echo " Choose a command to run:"
	@sed -n 's/^##//p' $< | column -t -s ':' | sed -e 's/^/ /'


# DEV #############################################################################################

## withenv: 😭 CALL TARGETS LIKE THIS `make withenv RECIPE=dev.init`
withenv:
# NB: IT APPEARS THAT LOADING ENVIRONMENT VARIABLES INTO make SUUUUCKS.
# NB: THIS RECIPE IS A HACK TO MAKE IT WORK.
# NB: THAT'S WHY THIS MAKEFILE NEEDS TO BE CALLED LIKE `make withenv RECIPE=dev.init`
	test -e .env || cp .env.example .env
	bash -c 'set -o allexport; source .env; set +o allexport; make "$$RECIPE"'

## dev.init: 🌏 Initialize local dev environment
dev.init: install
	$(call pp,install git hooks...)
	cargo install cargo-watch
	cargo test
	cd scylla_pg_js && npm install


# TEST / DEPLOY ###################################################################################

## install: 🧹 Installs dependencies
install:
	$(call pp,pull rust dependencies...)
	rustup install "${RUST_VERSION}"
	rustup component add rust-src clippy llvm-tools-preview
	rustup toolchain install nightly
	rustup override set "${RUST_VERSION}"
	cargo install cargo2junit grcov
	cargo fetch

## build: 🧪 Compiles rust
build:
	$(call pp,build rust...)
	cargo build
	cd scylla_pg_js && npm run build

bundle:
	$(call pp,bundle npm packages...)
	cd scylla_pg_js && npm run bundle:packages -- "$VER"

## dev.run: 🧪 Runs rust app in watch mode
dev.run:
	$(call pp,run app...)
	cargo  watch -q -c -x 'run --bin scylla_pg_monitor'
## run: 🧪 Runs rust app
pg.monitor:
	$(call pp,run app...)
	cargo run --bin scylla_pg_monitor

## db.migrate: 🧪 Runs DB Migration
db.migrate:
	$(call pp,db migrate...)
	cargo run --bin db_migrate

## db.create: 🧪 Creates a new DB
db.create:
	$(call pp,db migrate...)
	cargo run --bin db_create
# cargo run --bin db_delete
truncate:
	$(call pp, truncate...)
	RUSTFLAGS="" cargo run --bin truncate

## lint: 🧹 Checks for lint failures on rust
lint:
	$(call pp,lint rust...)
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets -- -D warnings

## test.unit: 🧪 Runs unit tests
test.unit:
	$(call pp,rust unit tests...)
	cargo test

## test.unit: 🧪 Runs unit tests
test.component:
	$(call pp,rust component tests...)
	cargo test -- --include-ignored --test-threads 1
test.component.lib:
	$(call pp,rust component tests...)
	cd scylla_pg_js && npm run build && npm run test
test.pg.client:
	$(call pp,rust component tests...)
	cd scylla_pg_client && npm run build && npm run test

test.nightly:
	$(call pp,rust test.nightly...)
	cargo +nightly test --workspace --exclude scylla_pg_js

test.json:
	$(call pp,rust test.json...)
	cargo +nightly test --workspace --exclude scylla_pg_js -- -Z unstable-options --format json --report-time > coverage/test-report.json

test.load.add_task:
	$(call pp,run app...)
	cargo run --release --bin load_add_task

# make withenv RECIPE=test.load.lease_task workers=5
test.load.lease_task:
	$(call pp,run app...)
	cargo run --release --bin load_lease_task -- ${workers}

# make withenv RECIPE=test.load.get_and_lease_task workers=5
test.load.get_and_lease_task:
	$(call pp,run app...)
	cargo run --release --bin load_get_and_lease_task -- ${workers}
# PHONY ###########################################################################################

# To force rebuild of not-file-related targets, make the targets "phony".
# A phony target is one that is not really the name of a file;
# Rather it is just a name for a recipe to be executed when you make an explicit request.
.PHONY: build
