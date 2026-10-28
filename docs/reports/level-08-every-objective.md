# level-08-every-objective — 07000008 completable end to end on real input

Branch `s/level-08-every-objective` (from `origin/master` 3c312512). Task: make
`crates/nf-view/tests/walkreal_07000008_live.rs` walk the WHOLE mission from a fresh load on real input,
every objective the mission data lists asserted complete, ending on the level exit firing.

## The objectives the mission data lists, and what the current tests covered

The level's mission data lists exactly **TWO objectives** for 07000008
[L ee_start_05.bin @ 0x2a3e88 — 2 records; the states `Mission_MonitorObjectives` 0x1827d8 runs; the port's
verified table `crates/nf-game/src/mission/c07000008.rs` quotes them]:

1. **"Retrieve guidance chip from Safe Room"** — channel 0x4A, enabled by ch 0x49, no fail record.
2. **"Escape with Zoe in gondola"** — channel 0x4B, enabled by ch 0x4A.

What the tests covered before this task:

- Objective 1 was covered by `walk_the_gondola_to_the_briefcase_and_back` on real input: the intro plate
  (ch 0x49 announces it, state 3), the fusebox laser (ch 137), the safe bolts (134/135 → mux 136), the
  briefcase take (ch 0x4A → state 4).
- Objective 2 was covered by `walk_the_gondola_ride_to_the_end` — but that test **poked ch 0x4A** as a
  "stand-in" for the whole briefcase chain (mux #235 folds 0x4A AND the floor plate's ch 4) and its Hind
  fight **re-stood `sim.bond.pos` 2.5 u from the machine on every trigger pull**. Both are what this task's
  rules forbid.

So the remaining work was not "more objectives" — it was one continuous fresh-load walk with NO pokes, plus
the blockers the honest legs exposed.

## What changed

`walk_the_whole_mission_on_real_input` (the two old walk tests superseded by one; every assertion from both
is carried over — the spawn pin, the carried 07000007 objective folding on ch 0x49 [L record 2], the ambush
plates 42/43/44 firing on the walked descent, the plunge guard 0x7C, the arrival guard, and the mission end):

1. fresh load at the station spawn #0 [D]; the carried latches (05's 0x45/0x70, 06's 0x02, 07's 0x48) are
   the mission's ENTRY state — in play they ride the RamSave [M `Player_RamSave` 0x195670 /
   `Player_RamLoad` 0x1957a8] — set once at spawn, never during the walk;
2. the intro plate → ch 0x49 → the station-intro scene 0x06000077 runs → objective 1 announced;
3. the fusebox #219 laser burn → ch 137 (the wire filter keeps the laser, drops guns [M
   `Collide_FilterBullets` 0x1ece30:24-31]); the Secure Room door #122 E-opened (real use);
4. the safe bolts #207/#208 laser-burned → 134/135 → mux 136 → SP #286 SafePickUps;
5. the briefcase take → ch 0x4A → **objective 1 → 4**;
6. the descent WALKED back down (the reverse legs + the ride walk's path-2 link chain [D]) — the ambush
   plates fire on the way; the stairwell; the west ledge; the floor plate → ch 4;
7. mux #235 on the **real** ch 0x4A AND 4 → ch 1 → the ride script 0x0600004C (1420 ticks) carries Bond
   [M `Control_InheritVelocity` 0x133e88] → ch 83 at tick 1410;
8. the bottom cinematic 0x06000078 → ch 5 → the Hind #45 flies (state 1/2 [M `Copter_Update` 0x17dbb8]);
9. **the shoot-down on real aim and trigger**: Bond stands where the cinematic left him; the aim tracks the
   machine's live position every tick (the real look input); the walk first switches to the gun the carry
   actually owns — the P2K 0x02 — through the real weapon-cycle input (the cycle is unlocked by the
   carried ch 0x48 [M `Player_Weapon` 0x1a24c8:242-275]); **27 trigger pulls, damage 350 vs hp 299 → state
   4 → rec+0x72 = ch 0x4B** [M 0x17dbb8:277-353 the part-contact damage walk, 0x17e390-0x17e3c8 the death
   tail's out write] → **objective 2 → 4**;
10. ch 0x4B → mux #276 → ch 100 → `Mission_Update` ends the level into the DRIVING hand-off **0x0730008**
    [M 0x182c60:85-116]. Total 32460 ticks, no falls, no stuck events.

## The blocker the honest fight exposed (and what it is)

With the old re-stand removed, the walk arrives at the fight with the **laser (0x4E)** equipped — and the
laser's beam deals **zero** damage to the Hind at the route's 24–37 u (45000 pulls, damage 0.0): the old
test's point-blank 2.5 u re-stand had masked what weapon the "shoot-down" really used. A player switches to
a gun; the carry owns 0x02/0x2C/0x4A [D inventory], and the P2K kills in 27 pulls. Whether the port's laser
should register copter part contacts at ANY range is **[S] unverified** — the game-side behaviour (does the
real laser beam damage the Hind on the disc?) is a live check below; it is not a pass/fail item, and the
level is completable without it (the gun is the real path).

No other blockers: the whole route walked with zero falls on the first honest run — the earlier legs' port
fixes (the doors, the fusebox, the locks, the ride, the copter) hold.

## What I ran

- `cargo test --offline --release -p nf-view --test walkreal_07000008_live walk_the_whole_mission_on_real_input`
  — green twice (the second run with the ambush assert carried over: **52.8 s**, `1 passed; 0 failed`).
- The binary compiles with the two unchanged ally tests (`the_ally_never_shoots_bond_on_the_walk`,
  `the_ally_fights_only_the_guards_through_the_noise_path` — untouched, still master's pins).
- The full four-crate suite via `tools/test/suite.py` on the final tree (started 19:30 UTC; NF_ISO the
  on-foot slim image) — see Suite.

## Suite

- Build: `build ok: 882 test binaries in … s` (warm target).
- Environmental on the slim ISO (the same four every session): `identity_disc` (wants the 1,191,072-sector
  retail image) + the 3 allowed `disc_golden` failures.
- The runner's total line: **(updated at the closing commit — see the final push's message)**.

## Live checks for the main agent

1. **The laser vs the Hind [S]**: on a fresh boot of 07000008, reach the bottom station with the laser
   equipped (the safe burns leave it in hand) and hold fire at the flying machine — watch the Hind's
   `rec+0x64` damage accumulator (`Copter_Update` 0x17dbb8:277-353) and whether any part sphere records a
   weapon contact for weapon id 0x4E. If the disc's laser cannot damage it, the port's zero is correct; if
   it can, the port's laser beam needs a contact path (a port gap to file).
2. **The carried latch set**: RamSave `switch_channels` at the 08 load [L] — the port's four channels
   (0x45, 0x70, 0x02, 0x48) should match byte-for-byte; any extra live channel on the real load is a carry
   the cold test should pin too.

## What's left

- The full suite's closing total line (the run was still in progress at the report's commit; the closing
  commit quotes whatever the runner printed — the partial per-binary results show 0 non-environmental
  failures through the binaries completed).
- The two live checks above (main agent).

## Usage

not visible to me — this environment shows no token, credit or context-window figures.
