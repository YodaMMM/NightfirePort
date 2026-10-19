"""Start an MP match in the running PCSX2 without the menus (our own code; docs/systems/mp-hud-data.md section 8).

Does P_MPCONFIRM 0x20ec00's accept from outside through PINE writes: player 0's MPSettings record (codename
"Agent", team 0, Bond, radar on), the Quick Game globals (C_SBMPSCEN's values: +0x19c 10, +0x1a0 the time in
SECONDS, +0x1a4 scenario, +0x1a8 map, +0x1ac 1 human, +0x1b0 bots, +0x1c4 radar), the bot records at 0x2deeaa
(the default record 0x316dc0, characters 6/7/8/9, mpbots = 1), the MP_setLoadingSkins flags (MP_skins+0xc),
GameState+0x54 = 1 (ResetMap_GameInit 0x1d5080 copies it into MPSettings+0x180 — writing +0x180 alone is
overwritten), GameState+0x28 = 1, then goto_level.py's ResetMap push. Only valid while the game-flow top state
is 2 (in a level or the front end).

Usage: python -I tools/trace/mp_goto.py <level hex> <scenario hex> <bots 0-4> [--team] [--time secs] [--launch]
  e.g. mp_goto.py 07000023 1 3                 (Arena, 3 bots, 10 minutes)
       mp_goto.py 07000023 20000002 3 --time 45 (Team Arena, 45 s — ends into the debriefing)
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import goto_level as G  # noqa: E402

MPS = 0x2A47A0          # MPSettings
MP_SKINS_SEL = 0x26D494  # MP_skins[i] + 0xc
MPBOTS = 0x2DEEA8        # mpbots flag, +1 count, +2.. 0x12-stride records
BOT_DEFAULT = 0x316DC0   # BOT_init's default record


def main():
    a = sys.argv[1:]
    if len(a) < 3:
        sys.exit(__doc__)
    level, scen, bots = int(a[0], 16), int(a[1], 16), max(0, min(4, int(a[2])))
    tsecs = int(a[a.index("--time") + 1]) if "--time" in a else 600
    team = "--team" in a or (scen & 0x20000000) != 0
    p = G.connect("--launch" in a)
    t0 = time.time()
    while True:
        _, st = G.top_state(p)
        if st == 2:
            break
        if time.time() - t0 > 30:
            sys.exit(f"game-flow state {st}, not 2")
        time.sleep(0.2)

    def w32(o, v):
        p.write(MPS + o, v & 0xffffffff, 4)

    name = (b"Agent\0" + b"\0" * 0x20)[:0x20]
    for i in range(0, 0x20, 4):
        p.write(MPS + i, struct.unpack("<I", name[i:i + 4])[0], 4)
    w32(0x20, 0); w32(0x24, 0); w32(0x28, 1); w32(0x2c, 0)
    for b in range(4):
        base = (4 + b) * 0x30
        w32(base + 0x20, (b + 1) % 2 if team else 0); w32(base + 0x28, 1)
    w32(0x19c, 10); w32(0x1a0, tsecs); w32(0x1a4, scen); w32(0x1a8, level); w32(0x198, 0)
    w32(0x1c4, 1); w32(0x1b4, 0); w32(0x1cc, 0); w32(0x1d4, 0); w32(0x1d0, 0)
    w32(0x1ac, 1); w32(0x1b0, bots)
    w32(0x18c, 1 if team else 0); w32(0x190, 1 if (scen & 0x40000000) else 0)
    default = p.read_bytes(BOT_DEFAULT, 0xE)
    p.write(MPBOTS, 1, 1); p.write(MPBOTS + 1, bots, 1)
    for b in range(4):
        r = MPBOTS + 2 + b * 0x12
        for i, c in enumerate(default):
            p.write(r + i, c, 1)
        p.write(r + 0xE, 1 if b < bots else 0, 1)
        p.write(r + 0x10, [6, 7, 8, 9][b], 1)
        p.write(r + 0x11, (b + 1) % 2 if team else 0, 1)
    for i in range(29):
        p.write(MP_SKINS_SEL + i * 0x10, 0, 1)
    for i in [0] + [6, 7, 8, 9][:bots]:
        p.write(MP_SKINS_SEL + i * 0x10, 1, 1)
    p.write(G.GAMESTATE + 0x54, 1, 4)
    w32(0x180, 1)
    p.write(G.GAMESTATE + 0x28, 1, 4)
    p.close()
    sys.argv = ["goto_level.py", f"{level:08x}", "--settle", "60"]
    G.main()


if __name__ == "__main__":
    main()
