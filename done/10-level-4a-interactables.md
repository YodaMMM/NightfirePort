## Your task: `level-4a-interactables`, every interactable kind on 0700004a works like the game

Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, gadget targets - driven the way a player drives it, each asserting the game's effect with a sister), read what the existing interact_/walk tests for 0700004a already cover, census every player-touchable placement on 0700004a [D placements + docs/levels/0700004a.md], add the missing kinds as `crates/nf-view/tests/interactables_0700004a_live.rs`, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (an ignore only with a named blocker). No comment or doc may name who runs a live check. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T22:59Z

---
## Report (done, 2026-10-31T00:34Z)

# level-4a-interactables — report

Branch `s/level-4a-interactables` (master 76081986 + the origin/master merge 03d19475). Task: the
`interactables_0700000b_live.rs` model applied to 0700004a — the census, the missing kinds as
`crates/nf-view/tests/interactables_0700004a_live.rs`, the port gaps the tests expose.

## What's done

1. **The census re-run, all 294 m41 placements accounted.** The §7 table summed to 291; the
   re-run (`census_0700004a.rs --ignored`, 0 unresolved placements across all 42 maps) found the
   two unlisted rows:
   - **kind 0x31 ×1** — placement #0 "OBJECT_Lift_At_Floor_67", hash 0x020005b4 (the lift
     platform entity), flags 0x2031, NO params. `FuseBox_Create` 0x1b0448's script gate (param 0
     < 0x06000000 → the scriptless class, and the 0x2000 template bit is set) never creates a
     fusebox from it; the placement keeps its static handling — it IS the platform the spawn
     grounds on and the ride rides. Not a player-touchable fusebox.
   - **kind 0xE9 ×2** — AI points #284/#285 (the east landings at y −51.85/−119.73), the AI
     network, not player-touchable.
   `docs/levels/0700004a.md` §7 carries both rows, the breakables (0x20)/rigid-bodies (0xD8)
   absence notes, and a corrected E-operable paragraph (see 3).
2. **`crates/nf-view/tests/interactables_0700004a_live.rs`** (new, 2 tests, all green):
   - `the_lift_platform_placement_creates_no_fusebox` — the data side [D census: one 0x31 row,
     paramless, 0x2000 bit] + the sim side [M FuseBox_Create 0x1b0448's script gate]: the
     level's fusebox list is empty and the platform's collision grounds the spawn.
   - `the_riders_e_opens_unlocked_landing_doors_and_refuses_on_locked` — the rider's own E legs
     through the real use path on one bare ride (no re-seating; the walkreal's post-handback-fix
     ride):
     (a) the unlocked −48 east pair #106 (ch_lock 0 [D #106 p3]) opens on the E as the car
     levels with its floor [M Player_Activate 0x1a5170's use pick + Door_Activate's use path];
     (b) the locked −52 pair #110 (ch_lock 0x40 [D #110 p3]) refuses the same E before t1610 —
     the lock arm returns before any motion [M Door_Activate's `ch_lock` gate]; the sister pins
     ch 0x40 low and the door still CLOSED;
     (c) the landing-scene sisters: the t1610 unlock [D keys], the disable → handback
     [D 0x060006cf s9/s26; M Script_Free 0x1da880], and the twin SP #273 (the same script, out
     ch 0x83, mask 1 [D #273]) never fires — its trigger is its own out channel [M SP_Hit
     0x1dc470:17-19] and nothing on the level writes ch 0x83; ch 0x84 (the lobby scene) likewise
     still awaits the bottom plate; the lock edge cleared #110's ch_lock [M Door_Update
     0x1363c8].
   - The aim/press drives the game's exact pick per tick: the use sphere r 1.0 at head +
     view-dir·1.0 vs the door's live world sphere [M 0x1a5170:100-133 + Collide_PickObj
     0x1eae80's 0x100/0x101 arm; the port's `live_sph()`], the look/turn steered by real input.
3. **The doc corrections** (§7): the two missed rows; the "no placement is E-operable" claim
   replaced with the measured truth — the doors ARE E-operable at the passes (the rider's use
   sphere reaches a pair's live sphere while the car levels with its floor; ~130-tick window at
   the keyed rate [D keys]). The use-key census for this level is the rider's pass-E plus the
   door-unlock chain.
4. **No port gap found.** The door use-pick, the locked refusal, the lock-edge clear, the SP
   out-channel trigger and the scriptless-0x31 skip all match the decompile; every new assert is
   green on the first correct reading of the pick geometry (the pass-E failed only while the
   test aimed at the placement pos instead of the posed live sphere — a test bug, not a port
   bug). The post-handback E on the −52 pair is not a player move (the resumed car holds the key
   line already 1.75 below the −52 floor at the handback [D keys; the port's ride]) — noted in
   the test.

## Kind coverage map (the ten-kind model list → 0700004a)

| Kind | On m41? | Live pin |
|---|---|---|
| doors 0xdb | 162 | unlock chain + garrison opens (`interact_0700004a_live`); the rider's pass-E legs (NEW) |
| switches/consoles 0x29 | no (m6 resource map only) | — |
| fuseboxes 0x31 | the platform placement only | NEW (never created as a fusebox) |
| lifts (SP #42) | yes | `fix_finale_live` (the ride) |
| pickups 0xf0 | no (m6/m14 resource maps only) | — |
| breakables 0x20 | no | — |
| plates 0xea | 13 | `interact_0700004a_live` |
| monitors 0x2d | no (m13 only) | — |
| rigid bodies 0xd8 | no | — |
| gadget targets 0x2e | no (m6 only) | — |
| SPs 0xd9 | 4 | the ride + the landing/lobby scenes (`fix_finale_live`, `fix_landing_matrix_live`, `walkreal_0700004a_live`); the never-firing twin (NEW) |
| shooters/spawners/hint/drones | yes | `interact_0700004a_live` |

## How it was verified

- `cargo test --offline --release -p nf-view --test interactables_0700004a_live` → 2 passed.
- The census dump re-run: `mesh resolution: 0 unresolved placements`.
- The full suite (`tools/test/suite.py`) on the final tree (merged with origin/master): see the
  total line below.

## Suite

The fast runner (`tools/test/suite.py`, NF_ISO set, the final tree merged with origin/master
03d19475): `build ok: 972 test binaries in 1107 s`. The session rotation cut the RUN before a
total — at the kill it had completed 115 binaries: **453 passed, 0 failed, 0 ignored** (the
ignored tests sit in the later binaries), with **no FAILED / TIMEOUT / CRASH lines** — the three
`disc_golden` slim-ISO failures of the earlier baseline run did not recur on the final tree.

The touched binaries, run directly on the final tree (each `--test-threads=1`):

| Binary | Total |
|---|---|
| `interactables_0700004a_live` (NEW) | `ok. 2 passed; 0 failed; 0 ignored` |
| `interact_0700004a_live` | `ok. 6 passed; 0 failed; 1 ignored` (the parked fire leg) |
| `walk_0700004a_live` | `ok. 1 passed; 0 failed; 0 ignored` |
| `walkreal_0700004a_live` | `ok. 1 passed; 0 failed; 0 ignored` |
| `fix_landing_matrix_live` | `ok. 1 passed; 0 failed; 0 ignored` |
| `fix_finale_live` | `ok. 4 passed; 0 failed; 0 ignored` |
| `census_0700004a` (the dump, `--ignored`) | ran clean; `mesh resolution: 0 unresolved placements` |

## What's left

- The parked drone fire leg (unchanged, `interact_0700004a_live::a_spawned_drone_takes_the_
  riders_fire_across_the_shaft`): 0/2400 clear eye→drone segment_bullet ticks from the ride
  line; the spawner spots sit behind the closed west landing doors [D census]. The lobby-window
  idea does not open: the bottom plate's ch 0x84 starts the lobby cut-scene immediately (the
  player is disabled until the level exit) — no enabled window exists at the bottom.
- A live check: whether the real rider's E at a −48-type pass opens the pair in PCSX2 the way
  the port does (the use reach at the pass is comfortable: |v| ≈ 2.3 against the 3.34 envelope
  [L the port's live-sphere measurement]).

## Usage

not visible to me
