ROM="./rscs/tetris.gb

test:
	cargo run --release -- $(ROM) -t 0

ctest:
	./rscs/tester.py

debug:
	cargo run --release -- $(ROM) -f debug 2> /dev/null

help:
	cargo run --release -- -h

.PHONY: test debug help ctest
