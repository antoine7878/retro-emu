ROM="./rscs/gb-test-roms/cpu_instrs/cpu_instrs.gb"

test:
	cargo run --release -- $(ROM) -t 0

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help
