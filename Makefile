ROM="./rscs/gb-test-roms/cpu_instrs/cpu_instrs.gb"
LOG_FILE=doctor.log

test:
	cargo run --release -- $(ROM) -t 0

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

doctor:
	rm $(LOG_FILE)
	gtimeout --foreground -k 2 2 cargo run --release -- -t 0 -d $(LOG_FILE) $(ROM) 2> /dev/null || true
	./gameboy-doctor/gameboy-doctor $(LOG_FILE) cpu_instrs 2

help:
	cargo run --release -- -h

.PHONY: test debug doctor help
