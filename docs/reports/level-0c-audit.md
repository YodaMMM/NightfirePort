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

Suite partial at finish: (filled in at finish.)

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
