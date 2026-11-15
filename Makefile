ROM=./rscs/games/pokemon_silver.gbc

test:
	cargo run -- $(ROM)

ctest:
	./tests/tester.py

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

roms:
	$(MAKE) -C roms

.PHONY: test debug help ctest roms
