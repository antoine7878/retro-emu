# ROM=rscs/tetris.gb
ROM=./rscs/gameboy_button_test/rom/buttontest.gb

test:
	cargo run --release -- $(ROM) 

ctest:
	./rscs/tester.py

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help ctest
