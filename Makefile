ROM="./rscs/gb-test-roms/instr_timing/instr_timing.gb"

test:
	cargo run --release -- $(ROM) -t 0

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help
