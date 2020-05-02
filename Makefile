readme:
	grep -E '^//!' src/lib.rs | sed 's/\/\/!\s\?//g' > README.md
	echo >> README.md
	cat ROADMAP.md >> README.md
