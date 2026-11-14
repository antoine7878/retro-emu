#!/usr/bin/env python3

import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import IO

ROOT = Path(__file__).resolve().parent.parent
TESTS = ROOT / "rscs"
MOONEYE_OK = bytes([3, 5, 8, 13, 21, 34])
MOONEYE_KO = bytes([0x42] * 6)
BIN = ROOT / "target" / "release" / "retro_emu"
TIMEOUT = 2
JOBS = 10
ROMS = [
    "gb-test-roms/interrupt_time/interrupt_time.gb",
    "gb-test-roms/halt_bug.gb",
    "gb-test-roms/instr_timing/instr_timing.gb",
    "gb-test-roms/cpu_instrs/cpu_instrs.gb",
    "gb-test-roms/cpu_instrs/individual/01-special.gb",
    "gb-test-roms/cpu_instrs/individual/02-interrupts.gb",
    "gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb",
    "gb-test-roms/cpu_instrs/individual/04-op r,imm.gb",
    "gb-test-roms/cpu_instrs/individual/05-op rp.gb",
    "gb-test-roms/cpu_instrs/individual/06-ld r,r.gb",
    "gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
    "gb-test-roms/cpu_instrs/individual/08-misc instrs.gb",
    "gb-test-roms/cpu_instrs/individual/09-op r,r.gb",
    "gb-test-roms/cpu_instrs/individual/10-bit ops.gb",
    "gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb",
    "gb-test-roms/mem_timing/mem_timing.gb",
    "gb-test-roms/mem_timing/individual/01-read_timing.gb",
    "gb-test-roms/mem_timing/individual/02-write_timing.gb",
    "gb-test-roms/mem_timing/individual/03-modify_timing.gb",
    "gb-test-roms/mem_timing-2/mem_timing.gb",
    "gb-test-roms/mem_timing-2/rom_singles/01-read_timing.gb",
    "gb-test-roms/mem_timing-2/rom_singles/02-write_timing.gb",
    "gb-test-roms/mem_timing-2/rom_singles/03-modify_timing.gb",
    "test-roms/mooneye/mbc1/rom_512kb.gb",
    "test-roms/mooneye/mbc1/ram_64kb.gb",
    "test-roms/mooneye/mbc2/ram.gb",
    "test-roms/mooneye/mbc2/ram.gb",
    "test-roms/mooneye/mbc5/rom_2Mb.gb",
    "test-roms/mooneye/acceptance/div_timing.gb",
    "test-roms/mooneye/acceptance/intr_timing.gb",
    "test-roms/mooneye/acceptance/oam_dma/basic.gb",
]


def run(rom: Path) -> str:
    proc = subprocess.Popen(
        [BIN, rom, "-t", "0", "-s"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    out = bytearray()
    stdout = proc.stdout
    assert stdout is not None

    def reader(stream: IO[bytes]):
        while b := stream.read(1):
            out.extend(b)

    t = threading.Thread(target=reader, args=(stdout,), daemon=True)
    t.start()
    deadline = time.monotonic() + TIMEOUT
    status = "TIMEOUT"
    while time.monotonic() < deadline:
        text = out.decode(errors="replace")
        if "Passed" in text or MOONEYE_OK in out:
            status = "OK"
            break
        if "Failed" in text or MOONEYE_KO in out:
            status = "KO"
            break
        if proc.poll() is not None:
            t.join(0.5)
            status = "CRASH"
            break
        time.sleep(0.05)
    proc.kill()
    proc.wait()
    return status


COLORS = {"OK": "\033[32m", "KO": "\033[31m"}


def remove_saves():
    Path("save_mooneye-gb test").unlink(missing_ok=True)


def main():
    subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)
    remove_saves()

    roms = [TESTS / r for r in ROMS]
    with ThreadPoolExecutor(JOBS) as ex:
        results = list(ex.map(run, roms))

    passed = 0
    for rom, status in zip(roms, results):
        passed += status == "OK"
        color = COLORS.get(status)
        label = f"{color}{status:8}\033[0m" if color else f"{status:8}"
        print(f"{label} {rom.relative_to(TESTS)}")
    print(f"\n{passed}/{len(roms)} passed")
    sys.exit(passed != len(roms))


if __name__ == "__main__":
    main()
