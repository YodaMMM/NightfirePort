## Your task: `mine-bob-41`, 0xFE mines: the bob is not drawn

`Mine.posed_y` (the 0xFE mine's bob, [M Mine_Update 0x17c930]) is computed but no draw path reads it; 07000015's mines all have zero bob, 07000041's were never checked. Census every level's 0xFE placements for a non-zero bob parameter [D], and if any bob, drive the mine's posed slot (scene.rs `mine_placements` / the mine slot block) with `posed_y` so the drawn mine moves as the game's does; pin it (sister: a zero-bob mine stays put). Run every *07000015* / *07000041* binary, ambient_sweep_live, load_scene_equality and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mine-bob-41`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T22:43Z

---
## Report (done, 2026-11-02T00:30Z)


---
## Report from the worker (done, 2026-11-02T00:30Z)

# mine-bob-41 — the 0xFE mine's bob is drawn

**Status: done.**

## What the game does (evidence)

- `Mine_Update` 0x17c930 writes the bob: `obj+0x34 = inst+4 +
  sin(GameState+0x52 · inst+0x14) · inst+0x18` [M disasm 0x17c9a8-0x17c9f4: both branches load
  `inst+0x14` into the sin's rate argument (`lwc1 $f12, 0x14($s0)` at 0x17c9a8/0x17c9c4), the
  product hits `PS2Sinf` (0x17c9c8-0x17c9cc), `inst+0x18` scales (0x17c9d0-0x17c9d8), `inst+4`
  is the base (0x17c9d4), the store is `swc1 $f1, 0x34($s2)` at 0x17c9f4].
- `Mine_Create` 0x17ce28 fills the instance from the placement's parameter records: inst+0x10 =
  param2·0.0002, **inst+0x14 = param3·0.0002**, inst+0x18 = param4·0.1 [M decompile 0x17ce28:45-47
  + disasm: the reads are `0x2c($s2)`..`0x40($s2)` of the scratch the dispatcher's caller fills by
  scattering the placement's (index,value) pairs to `scratch+0x2C+4·index` [M
  `parsemap_block_map_data_dynamic` 0x1d1b68:62-96] — so the port's `param(k)` lookup over
  `Placement::params` is the right source].
- `Mine_Update` also accumulates `obj+0x54 += inst+0x10·FRAME_RATE_MUL` [M disasm 0x17c978-0x17c988]
  — the **spin** angle (obj+0x50/0x54/0x58 are the object's euler angles: the movement handler wraps
  them into ±π [M `control_movement_object_handler` 0x1336f8:156-208]) — and sets `obj+0xFA |= 0x20`
  [M disasm 0x17c9dc-0x17c9ec].
- The drawn pose: the mine draws its glist at obj+0x90 [M `View_DrawObjectGlist` 0x1e6780:44,82];
  `Control_BuildWorldSph` 0x133070:17 rebuilds obj+0x90 every tick via `Control_GetObjMatrix`
  [M 0x134210:10-12: with `obj+0xFA & 6 == 0` — a fresh object is memset 0 and the mine only ever
  sets 0x20, so the RotTransMatrix arm holds; the handler calls BuildWorldSph on every 0x20-flagged
  dynamic object [M 0x1336f8:209]] from obj+0x50 (rot) + obj+0x30 (pos); `View_RotTransMatrix`
  0x1e5168:46-48 writes obj+0x30..0x38 into the drawn matrix's translation. So the drawn mine rides
  the bobbed obj+0x34 (and the spun obj+0x54).

## What changed in the port

1. **`Mine::bob_rate` now uses param3, not param2** (`crates/nf-game/src/worldobj.rs`). The port had
   the y-bob clocked at param2·0.0002 — that parameter actually feeds the obj+0x54 spin
   accumulation [M 0x17c930 disasm 0x17c978-0x17c988]; the sin's rate argument reads inst+0x14 =
   param3·0.0002 [M 0x17c9a8-0x17c9cc]. No test had pinned the old value (nothing contradicts).
   The formula shape `pos.y + sin(tick·rate)·amp` over the fixed 60 Hz frame is unchanged (the
   game's clock is its global `GameState+0x52` tick; the phase offset between the two clocks is a
   port choice, `// [S] unverified` — the pins below are phase-relative).
2. **The draw reads it** (`crates/nf-view/src/scene.rs`, the mine arm of the posed-slot chain):
   the slot previously returned the placement matrix for the mine's whole life; it now takes the
   sim's live `Mine.posed_y` into the matrix's y translation (keeping the placement rotation, which
   is the game's obj+0x50..0x58 at create [M `control_create_object` 0x1344e0:24-27 copies the
   create rot into obj+0x50..0x5C]), and collapses when the mine explodes exactly as before.

## The disc census [D]

`crates/nf-view/tests/mine_bob_live.rs::the_disc_census_of_0xfe_mines` walks every `.bin` archive in
FILES.BIN, every map, every 0xFE placement and pins the full result:

- 07000015 map 54: the nine tripbombs, p3 = p4 = 0 — bob off [D; matches docs/levels/07000015.md].
- 07000041 map 99: Laser_Mine #582 at (52.40, −8.52, −41.43), params [0x32, 0xa, 0x32, 0x32, 0xa,
  0x0600052] — p3 = 0x32 → bob rate 0.01 rad/tick, p4 = 0xa → amplitude 1.0 [D; matches
  docs/levels/07000041-audit.md line 455].
- 07000041 map 9 (a non-main map): one null 0xFE record (#0 at the origin, every param 0) [D]. The
  port's sim creates mines from the main map only, so this record makes no object and draws nothing.
- No other level or map carries a 0xFE placement [D]. (07000041 was never censused before this task;
  the "07000015's mines all have zero bob" premise held.)

So 07000041 is the one level where the drawn bob matters, and 07000015 is the zero-bob sister.

## Pins

`crates/nf-view/tests/mine_bob_live.rs` (all three pass):

- `the_07000041_laser_mine_bobs_in_its_draw_slot`: after k ticks the posed slot's matrix
  y equals `pos.y + sin(k·0.01)·1.0` (1e-4) for k = 1..260, the sim's `posed_y` equals the same
  [M] formula, the excursion tops +0.99 (the 1.0 amplitude [D] shows; the first quarter period is
  tick 157), and the static `mine_placements` matrix stays at the placement.
- `the_07000015_mines_stay_put_in_their_draw_slots` (sister): all nine slots keep the placement y
  for 120 ticks [D bob off].
- `the_disc_census_of_0xfe_mines`: the census above [D].

## What was run (totals)

`suite.py` (which wraps `cargo test --offline --release` per binary with `--test-threads=1`) could not
complete in this sandbox: its single build of all ~1,000 test binaries filled the 10 GB disk twice
(BUILD FAILED on ENOSPC). I ran the same commands as three space-budgeted groups on the final tree
(origin/master was 0 commits ahead of the branch cut, so the branch tree is the merged tree), same
flags, `NF_ISO` set, `NF_SILENT=1`:

- **The required nf-view binaries** (one `cargo test -p nf-view --test …` per name below,
  `--test-threads=1`):
  - `ambient_sweep_live`: **5 passed**, 0 failed
  - `census_07000015`: 0 passed, 0 failed, 3 ignored (assertion-free probes by design)
  - `census_07000041`: 0 passed, 0 failed, 1 ignored (probe)
  - `interactables_07000015_live`: **15 passed**, 0 failed
  - `interactables_07000041_live`: **14 passed**, 0 failed, 2 ignored
  - `interactables_07000041_probe`: **1 passed**, 0 failed
  - `load_scene_equality`: **1 passed**, 0 failed
  - `mine_bob_live` (new): **3 passed**, 0 failed
  - `walk_07000015_live`: **2 passed**, 0 failed
  - `walk_07000041_live`: **1 passed**, 0 failed
  - `walkreal_07000015_live`: **3 passed**, 0 failed
  - `walkreal_07000041_live`: **7 passed**, 0 failed
  - Group total: **52 passed, 0 failed, 6 ignored**.
- **nf-game** (all 100 of its test binaries, `cargo test -p nf-game -- --test-threads=1`):
  **1028 passed, 0 failed, 9 ignored**.
- **nf-formats + nf-audio** (`cargo test --no-fail-fast -p nf-formats -p nf-audio`):
  51 binaries: **229 passed, 4 failed, 3 ignored**. The failures:
  - the 3 `disc_golden` tests (`bigf_header_fields_behave_as_documented`,
    `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`) — the
    slim-ISO allowance ("only the 3 disc_golden tests may fail");
  - `identity_disc::the_real_disc_passes_the_identity_check` — a **pre-existing failure of this
    sandbox's ISO artifact, not of this tree**: the test hashes only the disc image itself against
    the retail identity (SHA-1 of ACTION.ELF, sector count), the `Nightfire-onfoot.iso` here is a
    derived/trimmed image that does not match the retail identity, and my diff touches no
    nf-formats code (the binary that failed was built from unchanged sources).
- Grand total: **1309 passed, 4 failed (3 allowed + the ISO-identity artifact), 18 ignored.**

The two earlier `suite.py` attempts are visible in the branch's `suite.out` history (not committed —
scratch output).

## What's left (not this task's bob, recorded for the merge)

- The **spin**: the game also rotates the mine around one euler axis at param2·0.0002·FRAME_RATE_MUL
  [M 0x17c930 disasm 0x17c978-0x17c988 + 0x1336f8's wrap]. The port models no spin; the drawn mine
  bobs but holds its placement rotation. 07000041's #582 has p2 = 0x32 (the same 0.01 rad/tick as
  its bob rate) [D], so the game's mine visibly spins there while the port's does not.
- The mine's world sphere / contact tris stay at the placement in the port (the game's
  `Control_BuildWorldSph` pose — and the sphere read from it — follows the bob). Untouched here.

## Live checks / renders for the main agent

The bob is visible work — render before merging:

- 07000041: the Laser_Mine sits at (52.40, −8.52, −41.43) [D]. A `--bond` spot on the order of
  `49.0,-6.5,-38.0,<yaw onto the mine>,<slight down pitch>` should show the mine ride ≈ ±1.0 m
  vertically with period ≈ 628 ticks (≈10.5 s) [M formula · D params] — and, in the real game only,
  spin slowly (the port's spin is the recorded gap).
- 07000015: any of the nine (e.g. #487 at (2.869, 41.986, 59.459) [D]) must sit perfectly still.

## Usage

not visible to me
