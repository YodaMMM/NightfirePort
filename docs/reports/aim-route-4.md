# aim-route-4 — the missile countdown is the game's 1/60 (REC_FRAME_RATE); the drakes walks' console phase is green on the true deadlines; the seeds' index map corrected; Drake's fight and the real-health clip are the next reds

Branch `s/aim-route-4` (from `origin/s/aim-route-3` f49ded65a; `origin/master` a6665d115 was
already an ancestor — the merge a no-op). Three commits: the countdown fix (085b88ca7, src),
the test re-pacing (bec12dff9), the drake_fight phase merge (8e4094c45). The [L] 08 pins and
the 0x30000000 fix are untouched; every other change is `space_missile.rs`'s constant + docs
and the three 0700001b test files.

## 1. The countdown fix (task item 1) — done, own commit

`COUNTDOWN_STEP` 512.0 → `REC_FRAME_RATE` = 1/60.0 [M the state-0 body: 0x1aa1f0 `lui
$at,0x31` / 0x1aa1f4 `lwc1 $f1,-0x2f24($at)` = the f32 at **0x30d0dc** (symtab
`0030d0dc` = REC_FRAME_RATE) / 0x1aa1fc `sub.s $f0,$f0,$f1` / 0x1aa200 `c.olt.s` / 0x1aa208
`bc1f`; the disarm writes at 0x1aa20c-238, base 0x30cd70 — disassembled from
`work/elf/ACTION.ELF` this session]. The old 512.0 cited the unmapped 0x31d0dc. The p3 seeds
are seconds, so each missile spends seed·60 gated ticks in state 0 and the eight state-0 exits
STAGGER.

**The seeds' index map corrected [D]**: the placement params re-dumped this session —
#129 105 · #131 105 · #132 40 · #133 20 · #134 0 · #135 60 · #136 105 · #137 60, i.e. by
missile index 1..8 **105/105/40/20/0/60/105/60**. The tree's "[D 105/40/20/0/60/105/105/60]"
list was mis-ordered against the p4 indices; two independent reads agree on the new map (the
param dump and the measured arming under a held gate: m5 at the first gated tick, m4 at +1200,
m3 at +2400 — the old map predicted m4 first, which the run disproved).

**What else the launch needs (the raw-ch19 mystery, pinned)**: ch 19 is the output of the
AND[193,194] mux and the mux writes it EVERY tick from its inputs [M triggers.rs's AND case] —
a raw ch-19 poke is overwritten by the mux's 0-write before the next gated tick, so the
countdown (which needs seed·60 GATED ticks) never runs. That is the PCSX2 observation ("a raw
channel 19 write started nothing in 118 s"): the poke held one tick. The drive that works
(measured: m5/m4/m3 armed at seed·60) pokes the mux's INPUTS 193/194 every tick; in the real
game the first wave's death latches ch 194 and the mux holds ch 19 forever after.

## 2. The true deadlines and the walk re-pace (task item 2) — the console phase is green

Console k's read deadline = launch + seed[k]·60 + P0_LEN[k−1] − 1 [M 0x1aa218/0x1aa23c-258;
D P0_LEN], with launch = ch19 at 2747 (unchanged — the wave death drives it):

| console | seed·60 | deadline (abs) | UI can first hold (armed+359 [D P0_GATE]) |
| --- | --- | --- | --- |
| k5 | 0 | 4546 | 3106 |
| k4 | 1200 | 5746 | 4306 |
| k3 | 2400 | 6946 | 5506 |
| k6 | 3600 | 8746 | 6706 |
| k8 | 3600 | 8746 | 6706 |
| k1/k2/k7 | 6300 | 12345 | 9406 |

`CONSOLE_ORDER` [3,4,5,6,8,7,1,2] → **[5,4,3,6,8,7,1,2]** (serves the deadlines weighted by the
UI-start times; k6 before k8 avoids a double crossing — k6 shares 3/4/5's x32 station), the
console phase cap 6000 → 12000. Measured run (`walk_arrival_to_drakes_defeat`, whole):

```
latch k5 3993 (−553) k4 5217 (−529) k3 6417 (−529) k6 7593 (−1153)
      k8 8055 (−691) k7 10233 (−2112) k1 10660 (−1685) k2 10803 (−1542)
MissileDeploy [1,1,2,2,2,2,1,2] → all-2 by ~12347 (m1/m7's reads at +9600)
```

**Every margin is negative (early) — the MissileDeploy gate that killed the last three windows
PASSED**, without any of aim-route-3's clip levers: the seed·60 stagger re-spaced the deadlines
more than enough. The walk then proceeded past the disarm chain (ch 46, ch 10), Drake spawned
at 13606 — and failed LATER, at **Drake's fight** (see §4).

## 3. The drive tests re-paced (they encoded the collapsed state 0)

- `drake_fight_live` (3/3 green): the disarm drive now shoots the held UI scripts from the
  FIRST gate (k5's at +361) instead of waiting for all eight (+6659) — on the disc clocks
  console k must latch inside its own window (m5's state-2 read at +1800 … m6/m8's at +6000);
  the fast-countdown all-armed MissileDeploy==1 pin is gone (replaced by the all-disarm pin).
  The fail-path test pins the new break shape: m5 (seed 0) fails the mission at ~launch+2801,
  deploy [0,0,1,1,3,0,0,0] at the fail, m4's ch 199 at ~+4201 inside the fail window.
- `interactables_0700001b_live` (9/9 green): the fail-window pins moved BEFORE the shots (the
  menu opens at fail+300 and the port's `fail_menu_open` pauses the player's input [M
  0x182c60:124]); the REAL shot moved to console 5 — the only console whose UI hold (~+1011)
  precedes the fail (~+4001 from m4); consoles 1/7/2 latch by the documented stand-in drives
  (their UIs hold at ~+7309, past the pause — the game itself cannot shoot them post-fail);
  the settled pin reads the staggered branches [2,2,3,3,2,3,2,3] with m5's done ch 25 up.
- The klaxon test's window grew to 14000 (0x50's starter chain now lands ~launch+9730) and its
  auto-restart is off (the fail at ~+7001 would otherwise reload the level mid-loop).

## 4. What's left (next window)

1. **The drakes walk's Drake fight** (both walks' remaining red, run 1's shape): Drake spawns
   at 13606 with the waves waited out, health 100/armour 0, and does not die inside the 6000
   -tick `fight_opts` cap. Never exercised before (every earlier window died at
   MissileDeploy). The clip state after the console phase and the float-fight aim pick are the
   first suspects; the fight's own 60 hp needs ~5-6 clean taps.
2. **The real-health walk starved consoles 7/1/2** (run 2: latched only 5/4/3/6/8, the loop
   expired with 7/1/2's UIs holding unanswered — the real-health fights drained the clip and
   the 48-burst gate re-armed late). The levers: the fight-exit clip top-up (aim-route-3 §3a)
   or a deadline-rationed burst gate (§3b) — with the true deadlines there is ~2100 ticks of
   slack on k7/k1/k2, so the rationing has room.
3. **08 x1 / 0c x1 / 16 x1** per aim-route-3 §4 — untouched this window (the 1b item consumed
   it). For 0c, check what Bond HOLDS first (pistol-class range 50 vs the d 61-90 snipers).

## Verification

- `interactables_0700001b_live` **9 passed / 0 failed** (whole binary).
- `drake_fight_live` **3 / 0** (whole). `space_laser_1b_live` **4 / 0** (whole; the missiles are
  inerted there, the ch-14 test's 700-tick window ends before any missile transition).
- `nf-game --lib` space_missile unit tests **6 / 0** (the crate's other 684 untouched).
- `walkreal_0700001b_live`: **15 passed / 2 failed / 2 ignored** — the two drakes walks (the
  console phase GREEN on the true deadlines, the failures per §4). The other 15 do not touch
  the edited fn.
- The full suite did NOT run (time): the branch touches `space_missile.rs` (read only by the
  four 1b binaries above, all run whole) and the three 1b test files; no `crates/*/tests/common/`
  change; no other level's walk reads the missile chain.

## Live checks

1. **The launch gate in the real game**: with the debugger, watch ch 193/194 and ch 19 after the
   first wave's death — confirm the AND mux holds ch 19 for at least 6300 ticks (the m1/m2/m7
   seeds' spend). Also: does the REAL game's fail flow (ch 99 → MissionState 2) keep the
   trigger/sound passes running? The port's `fail_menu_open` pauses the player input AND the
   trigger evaluation (measured: the mux-input hold stopped arming m1/m2/m7 post-menu; SP
   scripts kept running) — the game's GS_PausePlayer [M 0x182c60:124] may differ.
2. **The seeds on disc** (cheap confirm): read placements #129/#131-137's param 3 in PCSX2 —
   expect 105/105/40/20/0/60/105/60 by p4 order (the re-dump), and the first missile's state-0
   exit ~1 gated tick after the wave death (m5, seed 0).
3. **Drake's fight entrance** (carried): whether the real game's post-console waves spawn
   before Drake's gate at the disc timing — the port's run had Bond at health 100/armour 0 at
   Drake's spawn.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.

