# level-11-interactables-2 — 07000011: every player-touchable kind behaves like the game

**Status: done.**

## Method

The census of 07000011's placement kinds is `docs/levels/07000011-audit.md` [D — every placement
kind, per-map, with params]. Mapping the task's player-touchable kind list onto it:

| kind | placements on 07000011 [D] | live test |
|---|---|---|
| doors (0xdb) | 12 | covered (keypad/stairwell, plain+locked, storeroom plate) |
| switches/consoles | the Q-Worm PC (0x35 #216) + the keypad lock (0x2e #177) | covered (`qworm_computer_e_installs_and_writes_ch0x38`, `keypad_operative_unlock_holds_ch12…`) |
| fuseboxes (0x31) | 1 (#262, **no params at all** [D]) | covered (inertness is the only observable — see the sister fix below) |
| lifts | **0** | nothing to test |
| pickups (0xf0) | 13 | covered (`pickups_taken_by_walking_over`, medal gate sisters) |
| breakables (0x20/0xe1) | **0** | nothing to test |
| plates (0xea TouchOnce ×22 / 0xeb Touch ×2) | 24 | covered for the kind (storeroom plate, sound plates, walk/walkreal routes) — the Touch deassert + spawner-latch laws are newly pinned (below) |
| monitors | **0** | nothing to test |
| rigid bodies (0xd8) | **0** | nothing to test |
| SimpleScripts the player triggers | vent grills (0xe2 ×2, E), the Q-Worm NIS player (0xd9 #263), the SEQ mux (0xed #256) | vent/mux covered; **SP #263 was the gap — added** |
| (also touchable) exits (0xe8 ×2), hurt (0xe4 ×3), drone spawners (0xf1 ×7) | | exits covered (E arm + auto-fire arm + units); hurt covered; **spawners #220/#240 were never driven — added** |

## What was added (`crates/nf-view/tests/interactables_07000011_live.rs`, 14 → 16 tests)

1. **`scriptplayer_0263_plays_its_nis_from_load_and_holds_ch38_after_the_worm`** — the 0xd9
   ScriptPlayer kind. [M] `SP_CreateScriptPlayer` 0x1db9a8:84-86 loads the script at create for
   mode A ∉ {0,1}; `SP_LoadScript` 0x1dbc08's mode-8 arm `Script_Play(script, 1)`s it looping into
   state 6 — the NIS runs from level load. The out channel is written only on a mask match [M
   0x1dbda8:221-229], and `SP_Hit` 0x1dc470:15-17 builds flag bit 0 from the SP's OWN out channel —
   so with mask 1 [D #263 p5=1] the SP can never raise ch 0x38 by itself; it holds it only once the
   worm E writes it [M 0x1b0840; 0x1aee20 state 1]. Asserts: script hash 0x060008c0 [D], loaded +
   playing at the first ticks, ch 0x38 off before the E (sister), the real E at the PC, ch 0x38
   held 30 ticks after, the NIS still looping, the script's entities live. The port matched every
   law — **no port gap here**.
2. **`spawners_220_and_240_ignite_through_the_worm_and_the_touch_plate`** — the spawner kind's
   trigger arm on this level. [D #240 p1=3 p4=0x38 (start on the worm channel), #220 p1=2
   p4=0x35 (start on the mux out)]. [M 0x137e68:53-62 — a spawner sits dormant until its trigger
   channel fires and latches started]. Drives the real chain: the worm E → ch 0x38 → group 3
   starts (sister: group 2 does not); the touch #257 walk-in [D (−22.06, 47.45, 16.61)] → ch 0x39
   → the mux out 0x35 [M 0x1b2e08] → group 2 starts; walking out drops ch 0x39 and (with it)
   0x35 [M Touch's `out |= hit each frame`, the mux's combinational law] while the started
   spawner **stays latched** [M 0x137e68:62]. The port matched every law — **no port gap here**.
3. **The fusebox test's vacuous sister fixed** — the old line
   `assert!(…channels.on(0x12) || true, …)` could never fail. Replaced with a real Agent-load pin:
   20 idle ticks leave the lock's out 0x12 cold there [D #177 p6 = 1 (the Operative-only
   create-unlock tag); M `Lock_Create` 0x1ae1f8:67-69]. (The first attempt asserted `!on(0x12)` on
   the default Operative load and failed — correctly: the lock auto-unlocks at create on
   Operative, which is exactly what the keypad test pins. The test now documents both loads.)

No existing test contradicted the decompile; no port code needed changing.

## How it was verified

- [M] laws read fresh from the decompile this session (not from memory): 0x1db9a8, 0x1dbc08,
  0x1dbda8, 0x1dc470, 0x137e68 (quoted inline above), plus the audit's cited addresses for the
  already-covered kinds.
- [D] params from `docs/levels/07000011-audit.md` (placement tables quoted in the test comments).
- Every new assertion drives the real input path (the E key at the PC, the walk into the touch
  volume, the walk out), on real level loads (cold Operative load; the difficulty-3 Agent load for
  the spawner roster, the same calibration `spawner_kill_live` uses).

## What was run (totals)

- `interactables_07000011_live`: **16 passed, 0 failed** (was 14; two new kind tests).
- `walkreal_07000011_live`: **1 passed, 0 failed**. Also its neighbours on the same tree:
  `walk_07000011_live` **3 passed**, `fix_level_chain_live` **5 passed** (regression check over the
  exits/spawner chain the new tests touch).
- **nf-game: 1031 passed, 0 failed, 9 ignored** (all 100 of its test binaries,
  `cargo test --offline --release -j2 -p nf-game -- --test-threads=1`, NF_ISO set).
- nf-formats + nf-audio (the untouched crates, for the full-suite coverage):
  see the totals at the end of this file.
- The fast runner `tools/test/suite.py` was started (00:36) but killed and replaced by these
  per-crate runs: its single build of ~1,000 test binaries ENOSPCs this 10 GB sandbox disk (it died
  that way in the previous session too). The commands above are the same
  `cargo test --offline --release … --test-threads=1` the runner wraps, on the final tree (the
  branch is cut from origin/master 117f5b41 with no further master commits).

Final nf-formats + nf-audio line (51 binaries): **229 passed, 4 failed, 3 ignored** — the 3
`disc_golden` failures are the slim-ISO allowance
(`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
`directory_tree_matches_the_golden_manifest`); the 4th is
`identity_disc::the_real_disc_passes_the_identity_check`, a pre-existing failure of this sandbox's
ISO artifact (the test hashes the disc image itself against the retail identity; the derived
`Nightfire-onfoot.iso` does not match it, and this task's diff touches no nf-formats code).

Grand total this session: **1282 passed, 4 failed (3 allowed + the ISO-identity artifact), 12 ignored.**

## What's left

- Nothing for this task's scope. Notes for later tasks: the SP's touching-object flag bits
  (8, 0x10 — bullets/contacts on the script's entities) are not modelled in the port's `SP_Hit`
  stand-in (nis.rs documents the gap; no 07000011 placement needs them); the rotors' DRAW pose
  (the audit's separate draw-only gap) is untouched here.

## Usage

not visible to me
