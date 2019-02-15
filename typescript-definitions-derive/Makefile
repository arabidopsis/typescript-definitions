

test: readme
	@cargo test --all --features=test

doc: readme
	@rm -rf target/doc
	@cargo doc --no-deps --open
	# ./scripts/readme.sh

format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt

format-check: readme
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt -- --check

lint:
	@rustup component add clippy 2> /dev/null
	@cargo clippy

update-readme:
	@cargo readme > README.md

pre-publish: update-readme test format-check lint

publish: pre-publish
	@cargo publish

.PHONY:  doc test format format-check lint update-readme check-readme pre-publish publish
