ROM=./rscs/games/pokemon_silver.gbc

test:
	cargo run -- $(ROM)

ctest:
	./rscs/tester.py

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help ctest
