test:
	cargo test --all --features=test

doc:
	@cargo doc --no-deps 
	./scripts/readme.sh

