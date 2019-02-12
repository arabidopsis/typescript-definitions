.PHONY: doc

test:
	cargo test --all --features=test

doc:
	cargo doc --no-deps --open
	# ./scripts/readme.sh

src/README.rs : README.md
	@rm -f README.cache
	cargo docgen README.md > src/README.rs


readme: src/README.rs

