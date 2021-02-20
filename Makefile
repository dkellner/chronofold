default:
	cargo build

readme:
	grep -E '^//!' src/lib.rs | sed 's/\/\/!\s\?//g' > README.md
	echo >> README.md
	cat ROADMAP.md >> README.md

fmt:
	cargo fmt --all

test:
	cargo test --all-features
	RUSTFLAGS="-Dwarnings" cargo clippy
	cargo fmt --all -- --check

bench:
	cargo bench --all-features
