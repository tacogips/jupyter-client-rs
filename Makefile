TEST_FILTER:=

.PHONY: test
test:
	make test-jupyter-up
	make cargo-test

.PHONY: test-jupyter-up
test-jupyter-up:
	docker compose down
	docker compose up -d --quiet-pull

.PHONY: cargo-test
cargo-test:
	  cargo nextest run ${TEST_FILTER} --features=test_with_jupyter
