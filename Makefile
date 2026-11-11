# ROM="./rscs/tetris.gb"
# ROM="./rscs/gb-test-roms/instr_timing/instr_timing.gb"
ROM="./rscs/gb-test-roms/interrupt_time/interrupt_time.gb"

test:
	cargo run --release -- $(ROM) -t 0

ctest:
	./rscs/tester.py

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help ctest
