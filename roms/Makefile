BOOT  = dmg_boot.bin cgb_boot.bin
ERROR = no_game.gb bad_game.gb

all: $(BOOT) $(ERROR)

build/pb12: boot/pb12.c
	@mkdir -p build
	cc -std=c99 -Wall -Werror $< -o $@

build/SameBoyLogo.pb12: boot/SameBoyLogo.png build/pb12
	rgbgfx -Z -u -c embedded -o build/SameBoyLogo.2bpp $<
	build/pb12 < build/SameBoyLogo.2bpp > $@

%.bin: boot/%.asm build/SameBoyLogo.pb12
	rgbasm -I build/ -I boot/ -o build/$*.o $<
	rgblink -x -o $@ build/$*.o

no_game.gb: FIX = -m 0x1C

%.gb: error/%.asm
	@mkdir -p build
	rgbasm -I error/ -o build/$*.o $<
	rgblink -o $@ build/$*.o
	rgbfix -v -p 0 $(FIX) $@

clean:
	rm -rf build $(BOOT) $(ERROR)

.PHONY: all clean
