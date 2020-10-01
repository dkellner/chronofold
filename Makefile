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

bench:
	cargo bench --all-features
