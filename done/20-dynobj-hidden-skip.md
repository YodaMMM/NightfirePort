## Your task: `dynobj-hidden-skip`, placed dynamic objects the game hides from sight rays stop blocking them

Branch from origin/master. DynamicObject_Create 0x18dbf8:25-31 ORs 0x30 into obj+0xF0 when the placement's +0x2c short is non-zero; 0x20 is in every Collide_PickObj skip set [M 0x1eae80:70-82], so the game never picks those objects for any masked ray (guard sight, sensors, auto-aim, the beam reach). The port's world.rs Moving::pick_pair carries only the glist word plus the hide bit 0x10. Find which placements have +0x2c != 0 per level [D], carry the 0x30 into the dynamic object's flags, and pin it with a unit test (red on the old code) plus one real-level check where such an object sits on a guard's sight line, if any exists [D]. Run nf-game's unit tests, ai_live, and every walkreal_*_live binary one level per tool call.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-dynobj-hidden-skip`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T12:40Z

---
## Report (done, 2026-11-10T14:47Z)


---
## Report from the worker (done, 2026-11-10T14:47Z)

# dynobj-hidden-skip — report

Branch `s/20-dynobj-hidden-skip`, cut from origin/master 4bdaf3f50, then merged origin/master
20b5e52b5 (the final tree). Note: the prompt's header says branch `s/dynobj-hidden-skip` while the
task body says `s/20-dynobj-hidden-skip` — I used the latter (it matches the finish id
`20-dynobj-hidden-skip` and the repo's numbered branch convention, cf. `s/10-level-0b-roof-sprint-9`).

## What the game does (the proofs)

- `DynamicObject_Create` 0x18dbf8:15-16 copies the glist's +0x44 word into obj+0xF0; :24 reads
  `*(short *)(param_3 + 0x2c)` — param 3 is the `level_tag`, and per docs/fields/level-tag.md
  +0x2C is param 0 — and :30-32 OR 0x30 into obj+0xF0 when that SHORT is non-zero. [M]
- `Collide_PickObj` 0x1eae80:72-82 builds every query's skip word from a base of 0x20 (+0x1000 /
  0x40 / 0x10 by the query mask) and drops any object whose obj+0xF0 intersects it, for every
  query through the pick — so a 0x30 object is never picked by any ray, guard sight included. [M]
- `DynamicObject_Update` 0x18d3b8:9-15: a gated object (p0 = channel A) frozen while its channel
  reads off, and set to the record's RAW word (stored before the OR, 0x18dbf8:21-23) once it reads
  on; :36-43 mode 1: armed B re-ORs 0x30, disarmed restores the raw word. [M]

## What was already on master vs what I changed

Master already carried most of this: `worldobj.rs` ORs 0x30 at the 0x3b create (since the
world-objects port) and `drone.rs`'s dynamic-parts push (since 2998b6a76, 2026-11-08) filters out
every dynobj whose live flags carry 0x20, so hidden dynobjs have no collision part at all. The
task text's "world.rs Moving::pick_pair carries only the glist word plus the hide bit 0x10" is the
unmerged los-mask-envelope branch's shape, not master's. The real gap I found and fixed:

- **The create-time OR tested the full u32 param(0), not the game's short read.**
  `worldobj.rs` now gates it on `param(0) as u16 != 0` [M 0x18dbf8:24]. On real disc data this is
  a no-op (the census found zero placements where the two differ), but it is the game's condition.
- The dynamic object's flags and the 0x30 carry themselves were already correct; this task added
  the census and the pins.

## The [D] census (which placements have p0 != 0)

`crates/nf-formats/tests/dynobj_p0_census.rs` (opt-in NF_ISO, prints, asserts nothing):
**75 main-map 0x3b placements on the disc; 30 with p0 != 0; 0 u16-short disagreements; no
sub-map ones.** Per level:

- 07000007 — 2: pl 495 'OBJECT_RopeBarrier' p0=21 (0x15) at (80.48, 43.00, −50.49), glist word 0x0;
  pl 496 'Dynamic Blocker' p0=9 at (159.17, −1.15, −13.23), glist word 0x10 (sight-through base).
- 0700000d — 1: pl 113 'buttons_blink' p0=0x2b.
- 07000014 — 23: the CONSOLE_/SCREENS_ swap pairs (CONSOLE_Camera_Inactive p0=0x78 ×11,
  CONSOLE_Electricity_Inactive p0=0x8c, CONSOLE_Electricity_Static p0=0x5b,
  CONSOLE_Camera_Inactive p0=0x53, SCREENS_* p0=0x4e/0x5a/0x51/0x5c/0x78…) — each sits at the same
  spot as its p0=0 'Active' twin and shows when its channel turns on (the Update gate).
- 07000016 — 3: 'Door2_switch_locked' p0=0x94, 'Blocking_polys_lift_shaft1' p0=0xcc (glist word
  0x110 — the OR folds into the standing 0x10 bit → 0x130), 'Screen_door_unlocked' p0=0x24.
- 07000041 — 1: 'DynoRod' p0=1, glist word 0x880.

The runtime live flags (what the port's part filter reads) match: on 07000014 the Sim carries 45
dynobjs with shapes, 23 of them flagged 0x30 after the warm ticks.

## Tests (all red-on-the-old-code demonstrations included)

- **Unit** (`crates/nf-game/src/worldobj.rs` `dynobj_hidden_tests`, 5 green):
  - `create_or_the_hide_word_only_for_a_nonzero_p0_short` — the OR for p0 21/1, none for 0/absent,
    and **none for p0 = 0x10000** [M 0x18dbf8:24]. RED on the old u32 condition (demonstrated:
    scratch-reverting the one line fails exactly this test), green with the fix.
  - `the_or_folds_into_a_sight_through_base_word` — pl 496's word 0x10 | 0x30, ch_a = the same p0.
  - `gate_on_restores_the_raw_word_and_gate_off_freezes_the_create_word` [M 0x18d3b8:9-15].
  - `mode_one_toggles_the_hide_word_with_channel_b` [M 0x18d3b8:36-43].
  - `the_hidden_word_is_admitted_by_no_mask` — `pick_admits(mask, 0x30/0x20)` false for masks
    0..0x72f, the 0x10 word only skipped by bit-4 masks [M 0x1eae80:72-82].
- **Real-level check** (`crates/nf-game/tests/dynobj_hidden_live.rs`, green; opt-in NF_ISO): loads
  07000007 and 07000014 through the real loader, spawns the levels' real drones (57 level-start
  guards on 07), ticks 15 warm frames on the real Sim, then:
  - for every (guard vantage → hidden dynobj centre → 10 units past) ray that the PRE-FIX world
    stops inside the object's bounds (12 clean crossings on 07000007, e.g. guard 19 → pl 495:
    stops at t 0.245 in span [0.230, 0.259]) the fixed Sim's dynamic set does NOT stop the ray
    inside those bounds — the hard assert;
  - the sisters: the VISIBLE rope barriers on the same fence line (p0 = 0, word 0x0, slots 0-6) DO
    stop guard sight rays inside their own bounds — the data tells hidden from visible;
  - end-to-end red: with the drone.rs 0x20 filter scratched (the pre-2998b6a76 shape) the test
    fails — "the hidden dynobj slot 7 stopped a guard sight ray (t 0.5515 in [0.539, 0.564])".
  The guard vantage is guard pos + 0.9 [S — a stand-in head offset; the claim rests on the proven
  triangle crossing, not the eye byte]. **Such an object on a guard's sight line exists [D]: the
  07000007 rope barrier and Dynamic Blocker both cross real guards' vantage rays.**

## What I ran (each binary whole unless noted)

- nf-game lib: 778 passed / 0 failed / 5 ignored.
- nf-game ai_live: 37 passed.
- nf-game dynobj_hidden_live: 1 passed.
- nf-formats dynobj_p0_census: 1 passed (census prints).
- nf-view worldobj_live: 6 passed; nf-view dynobj_glass_live: 2 passed (both consumers of the
  world-objects code).
- walkreal_*_live (one per tool call): 07000001 7/1ign, 07000002 5, 07000005 16/1ign,
  07000007 6, 07000008 5/1ign, 07000009 9, 0700000a 13/2ign, 0700000b 8, 0700000d 1/2ign,
  07000014 7/3ign, 07000016 15/3ign, 07000041 15, 0700004a 4, 0700004a_satchel 2 — **all green**,
  including every level that carries hidden dynobjs (07, 0d, 14, 16, 41).
- Not run: walkreal_07000003_live whole (its last real-damage walk outgrew the 10-minute tool cap
  twice; the 5 tests before the cut passed both times, the interrupted one never finished), and
  the remaining walkreal levels (0700000c ×3, 07000011 + route, 07000012, 07000013, 07000015,
  0700001b, celprobe/dump/probe) — my only src change is provably a no-op for every disc p0
  (census: zero short disagreements), so these are not expected to move.
- **The fast-runner full suite does not fit this sandbox**: background jobs die between tool calls
  here (two `nohup python3 tools/test/suite.py` runs died with empty logs within seconds of the
  call ending; setsid does not survive either), and the foreground 10-minute tool cap killed the
  4-crate test build twice at ~10 min. The per-binary list above is what the window allowed.

## What's left

1. The fast-runner suite once on this final tree (merged with origin/master 20b5e52b5) — the next
   window can start it first thing; the branch is pushed and green per-binary.
2. walkreal_07000003_live and the unlisted walkreal levels, same expectation.
3. The `inside(o)` carve-out in drone.rs (a 0x10 part stays out while Bond is inside its box, the
   07000005 crawl-exit workaround) remains an [S] deviation from the game's rule — untouched here.

## Live checks for the main agent

1. Fresh boot 07000007, then fieldprobe the two dynobjs' obj+0xF0 right after the load: pl 495
   ('OBJECT_RopeBarrier', p0 short 0x15) should read 0x30 and pl 496 ('Dynamic Blocker', p0 short
   0x9) 0x10|0x30 = 0x30 — the create OR [M 0x18dbf8:24-32].
2. Same level, poke switch channel 21 on (and then 9): the gated object's obj+0xF0 should drop to
   its raw glist word (0x0 resp. 0x10) on the next tick [M 0x18d3b8:14-15] — i.e. the blocker
   becomes ray-pickable exactly when its channel turns on.
3. A guard sight check: from a guard on the 07000007 slope (the spawn cluster near (88, 45, −60))
   toward pl 495 at (80.48, 43.00, −50.49) — the game's vision should treat the barrier as
   transparent (no pick), matching the port's ray behaviour pinned above.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment).
