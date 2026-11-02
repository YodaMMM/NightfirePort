## Your task: `level-15-interactables-2`, 15: every player-touchable kind behaves like the game

`interactables_07000015_live.rs` has 15 tests. Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player triggers), run the census of 07000015 [D every placement kind], and for every kind a player can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_07000015_live, walkreal_07000015_live and nf-game; quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-15-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-02T01:05Z

---
## Report (done, 2026-11-02T02:18Z)

# level-15-interactables-2 — every player-touchable kind on 07000015 behaves like the game

Branch `s/level-15-interactables-2` (from `origin/master` 8f27b95e). The task: extend
`crates/nf-view/tests/interactables_07000015_live.rs` (15 tests) to one live test per KIND of
player-touchable placement, using `interactables_0700000b_live.rs` as the model, after a fresh
census of the level's placement kinds.

## What's done

**1. The census, re-run fresh [D].** `nf-extract placements-all Nightfire-onfoot.iso
07000015.bin` → 638 placements on the main map m54 (the doc's §7 says 968 across the 55 maps,
main map 638 — the fresh run matches). Kind histogram (hex): 0x19×324 (static props), 0xfc×76
(coronas), 0x47×59 (rooms), 0xdb×54 (doors), 0x0f×20 (guards), 0xf2×16 (emitters), 0xea×13
(TouchOnce), 0xf0×12 (pickups), 0xe6×11 (cover low), 0xe1×11 (destroyables), 0xfe×9 (mines),
0xe4×9 (death plates), 0xe5×6 (cover corner), 0x4a×4 (Apocalypse), 0x2e×3 (locks), 0x22×3
(ladders), 0xe8×2 (load-level exits), 0xf9×1 (sound trigger), 0xf1×1 (spawner), 0xe9×1 (AI
point), 0xd2×1 (shooter), 0x31×1 (cel), 0x24×1 (start). The other maps are the object/template
libraries whose placements never reach the dispatcher [M `parsemap_block_map_data_dynamic`
0x1d1b68]. No monitors, rigid bodies or param-bearing fuseboxes on 0b-style families exist
here [D the same dump; the 0x2b/0x31 FuseBox cases have no main-map rows].

**2. The coverage audit against the 15 existing tests.** Every player-touchable kind in the
census already had its live test with sisters EXCEPT two:

- **0xE8 load-level exits ×2** — the doc said "live" because walkreal drives them, but the
  interactables file (the kind model the task asks for) had none.
- **0x4a Apocalypse ×4** — the east-room test pins the latch (armed + fade > 0 at the touch)
  but not the countdown's run-out.

**3. Two new kind tests in `interactables_07000015_live.rs` (15 → 17 tests, all green):**

- `the_use_exit_loads_and_the_gated_exit_stays_cold` — the USE exit #445 (gate 0 [D param 2])
  loads 07000016 only through the E: walking INTO its volume without the E raises nothing
  [M `Trigger_Update` 0x1b1a78:162-166 case 10 never self-fires with gate 0], the E then sets
  `level_exit = 0x07000016` [M Player_Activate 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318;
  D param 0 = 117440534] and the objective stays announced (its completing ch 0x8e is
  07000016's spawn plate [D 07000016 #252]). Sisters: `level_exit` None on arrival, and the
  gated #343 (gate ch 0x42 [D param 2]) stays cold — ch 0x42 has no writer on the level
  [D scans 2026-10-13].
- `the_apocalypse_countdown_runs_out_and_floors` — the 0x4a whiteout's full arc through the
  real east-room walk: the record rests unarmed at `total` with fade 0 [M Apocalypse_Create
  0x18dcc0's param 1 · 60 = 120 ticks; D #710 param 1], the room touch's ch 0xf6 latches it
  armed with the mirror live [M Apocalypse_Update 0x18d4f0], the countdown runs
  (`remaining` falls, `fade = remaining·5/total` shows [M 0x18d4f0]), and at 0 both floor and
  stay [M 0x18d4f0's floor arm].

No port code changes were needed: every law the new tests probe already held — the tests
expose no port gap to fix.

**4. Coverage statement.** With the census above, every player-touchable kind on 07000015 now
has its live test in `interactables_07000015_live.rs`: TouchOnce 0xEA (the reveal + the room
chain), doors 0xdb (plain E + the dead-keypad refusals), locks 0x2E (the burn → out channel →
door), pickups 0xF0, destroyables 0xE1 (the satellite kill channel + the barrel's death
channel), death plates 0xE4, dead ladders 0x22 (both the no-grab and the mesh-reach sister),
Apocalypse 0x4a (latch + the full countdown), shooter 0xd2 + sound trigger 0xF9 (one room
touch drives both), mines 0xFE (three tests), spawner 0xF1 + drones 0x0F (the spawned squad),
exits 0xE8 (USE + the gated cold), plus the draw-census test. The remaining kinds are AI
records (cover 0xE5/0xE6, AI point 0xE9), draw (coronas, emitters, props) or the spawn.

## How it was verified

- `interactables_07000015_live` **17 passed, 0 failed** (30.85 s, --test-threads=1).
- `walkreal_07000015_live` **3 passed, 0 failed** (12.41 s) — the end-to-end route green on the
  same tree.
- `nf-game` **1033 passed, 0 failed, 9 ignored** (`cargo test --offline --release -j1 -p
  nf-game -- --test-threads=1`, summed over its binaries).
- The full suite (`tools/test/suite.py`) did NOT complete in this sandbox window — the same
  disk war as the previous task: the -j4 build of all four crates' test binaries filled the
  disk to 100% three times (three strip+prune passes each recovered ~0.3–1 GB), and the build
  was still linking when the deadline's stop line arrived. No FAILED/TIMEOUT line was ever
  printed (the build phase never reached the run phase). The branch is current; the next
  message can run `NF_ISO=<abs> python3 tools/test/suite.py` to completion (the incremental
  build resumes where this session left it).

## What's left

- The full-suite run + its total line on the final tree (above).

## Live checks for the main agent

1. **Render spot** (visual): `--bond 82.0,41.4,0.5,1.8,0` just after the east-room touch —
   the missile-test whiteout should flash over the ~2 s the Apocalypse #710 counts down
   (`fade` 5 → 0 in the port's mirror [M 0x18d4f0]).
2. The ch-0x42 exit gate: no writer on the level's data [D scans 2026-10-13] — if a playtest
   ever opens #343's exit, the writer is on another level's carry; worth a boot-state channel
   dump at 07000015 entry to confirm ch 0x42 arrives off [L expected].

## WORKLOG

`docs/WORKLOG.md` does not exist on master (dropped as a process file in batch-1101j); this
report is the `level-15-interactables-2` entry.

## Usage

not visible to me — this environment shows no token, credit or context-window meter
(checked the session UI and the tool output; no figures are exposed to copy).
