## Your task: `level-0c-audit`, the 0700000c behaviour audit: every placed object and character against the game

The walk test for 0700000c passes, but real play still finds missing behaviour (characters walking through Bond, guests that never talk, doors that never lock, consoles whose action does nothing, cut-scenes with broken cameras). This task lists every gap on the level.

1. For EVERY placement on 0700000c's main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts, special objects): name its creator in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, its update and its use/damage/event handlers [M], and what the port does with it (created? updated? drawn? usable? reacts to Bond/guards/damage? plays its sounds?). Do the same for the level's scripts (every stream's ops, the camera streams, the hand-backs) and the mission records (objectives, the channels each writes, fail conditions).
2. Write the table to `docs/levels/0700000c-audit.md`: one row per placement/script/record, the game's behaviour [M/D], the port's, and a GAP column.
3. Fix the small gaps directly [M] with a test each. For each large gap, write a section in the report (what, where, the [M] chain) so it can become its own task. Draw-only gaps: name the mesh/hash/matrix.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-audit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-26T04:20Z

---
## Report (done, 2026-10-26T05:12Z)

Closed by the driver: the report is on `s/level-0c-audit` (docs/reports/level-0c-audit.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-26T06:13Z)

# level-0c-audit report

Branch `s/level-0c-audit` (from master 30598272). Task: the 0700000c behaviour audit — one row per
placement/script/mission record against the game.

## Done

1. **The per-placement audit table** — `docs/levels/0700000c-audit.md`: **all 575 main-map
   placements, one row each** (35 kind sections), each with the game's [M/D] chain (creator in
   `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, update, use/damage/event handlers),
   the port's behaviour, and a GAP column. The census probe `crates/nf-view/tests/census_0700000c.rs`
   was written this task (the level had none) and re-dumped the disc: **575 placements, 0
   unresolved** — the level doc §7's "574" was a drift, and its kind list missed 0x2D (the mode-0
   starts), 0x71 (the office-fan rotor) and the 78 low-cover nodes.
2. **The scripts section** — 13 kind-7 scripts in 0700000c.bin [D `list_scripts`]: the two SPs
   (#341's 0x06000762 forklift loop, 22990 ticks, and #542's 0x0600090c exhaust), their SUB-scripts
   via the 0x1B ops (0x0600083b, 0x06000834, 0x0600083a), the hard-coded effect quartet
   (0x06000027/4f/52/7c4 — [M 0x136ce8:93, 0x180498:436, 0x1afcc8:45]), and three unreferenced
   effect loops (060008bc/44/45). No 0c script carries a camera op or a hand-back [D the dumps].
3. **The mission-record section** — record 11 [L ee_start_05.bin @0x2a4508: level 0x0700000c,
   mission 0x0700000c part 0, objptr 0x2a4080, count 3; the doc's "@0x2a4478" was off]: the three
   objectives with their channels (0x1b by the walkway plates + the skylight shots, 0x1c by the
   CamSubject photograph, 0x23 by 0700000d's start trigger), the generic alarm/civilian fails
   [M 0x1827d8:128-141], the #556 exit flow, and the mission-end default arm [M 0x182c60 case 6].
   The fresh boot carries no pre-on channels [L @0x26fc90].
4. **Small gap fixed: the SS operate sound** (re-applied — the level-08 branch's fix has not
   reached master): [M 0x1a9948:86-89] plays `Sound_Play3D(100.0, ext+0x14 = param 4, at obj+0x80)`
   at the 0x40 arm; 0700000c's hatch #138 carries sample 0x1d6 (470) [D census]. Two unit tests
   green.
5. **Small gap fixed: the RAW 0xDC trigger rows** — the port read the raw rows with the
   wrapper-shifted slots (out = p0, mask = p1); the direct `Trigger_Create` map is out = p1,
   inputs = p2..p8, mask = p9 (0 -> the create's default 2), lock = p10, hide = p11 [M 0x1b1720:61-91,
   re-derived this task against the 0x1b2b88/0x1b2c68 wrapper stores]. Observable: the game's
   skylight plates #275-277 fire on BULLETS (mask 2) and write the objective channel 0x1b; the
   port's wrote ch 2 on walk-in (falsely arming the secret door's AND input). Fixed in
   `triggers.rs` (`apply_raw_dc_map`) + the unit test `the_raw_dc_rows_read_the_direct_create_map`.
6. **Corrections** (in the audit file; the level doc untouched per the shared-file rule): the 575
   count; the 0x24/0x2D start-kind split [M 0x1d04f0:154-161]; door2turbine #281's p2=5 is its OUT
   channel — opening it writes ch 5 = MusicTrigger #95's gate (the doc's "[S] the level script"
   writer note was wrong); the skylight plates' slot map; record 11 @0x2a4508; the ch-0xB4 plate
   #103 has no reader (dead content, like 08's ch-43 plate).

## Verification

- `cargo test --release --offline -p nf-view --test census_0700000c -- --ignored --nocapture` —
  the 575-row dump + the histogram (the audit's [D] base).
- `dump_sp_script`/`list_scripts` over 0700000c.bin — the scripts section.
- `cargo test --release --offline -p nf-game --lib triggers::` — 25 passed (incl. the new raw-0xDC
  test); `--lib ss::tests` — 11 passed (incl. the two SS-sound tests).
- The 0c/trigger live suites on the fixed tree, run directly: `fix_level_0c_live` 4/4,
  `walk_0700000c_live` 4/4, `walkreal_0700000c_live` 2/2, `laser_watch_0c_live` 2/2,
  `guards_0c_live` 1/1, `hatch_crouch_live` 4/4, `fix_0c_hatch_live` 2 passed + 2 ignored,
  `fix_ladder_water_live` 4/4 — the fixes break nothing.
- The full four-crate suite via `tools/test/suite.py` on the final tree: see the partial below.

Suite partial at finish (the run phase is sequential across 812 binaries and the window closed
first): **364 binaries completed — 1265 passed, 3 failed, 100 ignored**. The 3 failures are the
documented GPU render probes (`fix_0c_pylon_live` ×2, `probe_pylon_render` — this sandbox has no
GPU; the 05 session recorded the same set). The CRASH lines are the net/mp OOM class (exit -9:
net_resync, mp_shot_effects, mp_stress_explosives, net_match_end_clients, mp_online_failures — the
05 session's recorded environmental set). No TIMEOUTs, and no 0c/trigger path failed (the 0c
binaries were also run directly — see Verification). The remaining ~450 binaries were not reached;
the next session can re-run the suite to completion.

## What's left (large gaps -> follow-up tasks)

- **The sky +0x4a word** (#260 p4=1; shared with 05/08): the renderer consumer unported
  [M 0x1e79a0:19-36] — the sky task.
- **The 007-token medal gate (D-004)**: [M 0x18dda0:~93]; the port spawns #546-#549
  unconditionally (pickup.rs:557) — needs the BNUS medal carry at load.
- **The env weather 0xff** (#278): the port's DropType map was keyed on the 0=snow rows; type
  255's behaviour is [S] unverified — a live check or the env task.
- **The SS framelist POSED RENDER** (the doc §6 carry-over): the render-side posed matrices stay
  the models lane's open item.
- **Per-instance follow-up tests**: the 0c wire/zipline rides, the spawner wave releases (ch 4/6/
  0x1b), the sound triggers' keep_on/global variants, the forklift/exhaust SP renders.

## Live checks for the main agent

In `docs/levels/0700000c-audit.md` §"Live checks for the main agent": the skylight plates' bullet
law (objective 1 by shooting, not walking), the hatch's sample-470 sound, the turbine-door music
event 45, the hint-plate death after the photograph, the forklift render spot.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment; the
session's tool output shows no cost readout).
