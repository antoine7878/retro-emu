ROM="./rscs/gb-test-roms/cpu_instrs/cpu_instrs.gb"
LOG_FILE=doctor.log

test:
	cargo run -- $(ROM) -t 0

debug:
	cargo run -- $(ROM) -f debug 2> /dev/null

doctor:
	rm $(LOG_FILE)
	gtimeout --foreground -k 2 2 cargo run -- -t 0 -d $(LOG_FILE) $(ROM) 2> /dev/null || true
	./gameboy-doctor/gameboy-doctor $(LOG_FILE) cpu_instrs 2

help:
	cargo run -- -h

.PHONY: test debug doctor help
