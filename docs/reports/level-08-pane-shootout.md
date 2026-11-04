# level-08-pane-shootout — the 08 bottom-terminal pane shoot-out and the Hind fight's rockets

Branch `s/level-08-pane-shootout`, cut from `origin/s/level-08-00agent-hind` and merged
origin/master (commit bd3ca3f5). **Status: DONE — both of the hind session's blockers root-caused
and fixed inside the walk, `walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent`
green and UN-IGNORED, the whole `walkreal_07000008_live` binary 6 passed / 0 failed / 0 ignored.**

## What was done

### 1. The pane shoot-out: the panes were never shot — the test's fire gate was the bug, not the sim

The hind session's orphaned-pane lead ("the shoot-out taps 9 rounds and kills NOTHING — every pane
reads OPEN") resolved to a test-side law, measured from the diagnostic runs:

- The old shoot-out tapped a pane only while its aim ray (eye → the pane bounds' CENTRE) read
  BLOCKED, and stopped the moment the ray read clear. But the captured centre is the pane bounds'
  midpoint, which sits a hair SHORT of the sheet plane — the north pane's centre ray reads OPEN
  before a single round while a perpendicular ray through the sheet hits at t 0.52 [L the probe
  raytest 2026-11-04] — so `done` held at tick 0 and the tap fired nothing. The 9 rounds of the
  hind session's log were the earlier fights'.
- The sim's pane-damage law needed NO change: the port's player-bullet pass already registers the
  seg_box_t contact over the pane bounds and drains the row (the game's mirrored contact on the
  glass object's obj+0xD0 chain, `Break_Update` 0x12bfd8:36-37 [M]; the port's contact in
  `crates/nf-game/src/drone.rs`'s break contact block) and a health-1.0 pane dies to one round
  [D the ride script's params [2,1,0]].
- The fix (in `crates/nf-view/tests/walkreal_07000008_live.rs`): the shoot-out aims at the pane
  centre with the real look input and pulls the trigger on a CONVERGED aim (the fire edge every
  12 ticks — a held trigger is one pull [L the pane hold]), up to 5 rounds per pane, until the
  row dies. **Measured: 10/10 end panes die to ONE round each (clip 20 → 11, spare 7), and both
  end bearings then read CLEAR 30 u** — a shot-out window breaks AND clears for the player, the
  task's own criterion. The walk asserts it: `unbroken == 0`.

### 2. The rocket self-blasts: the shot leaves through the crosshair, and the tracker parked the crosshair at its clamp

With the panes dead the fight still died — twice, on real input: rocket 3 (t347) detonated 2.4 u
from Bond (−75 [L the diag run — the world hit at frame 28066]) and a later launch 2.2 u (−62.5
[L frame 28470]). The chase-arc blocker, measured from the launch-tick and per-flight dumps:

- **The shot's direction is the crosshair's, not the view's**: aim = look + 0.76966·cx·right +
  0.57724·cy·up, normalized [M the crosshair pass]. The walk runs free-aim (`free_aim true` at
  every fire dump), and the walk's own tracker — converging with a sustained clamped turn —
  drives the free-aim cursor to its clamp and it never returns (the fire dump at every launch:
  cross (−0.296, ±0.394), cursor pinned at (−0.296, ±0.4)). Every launch therefore sat up to
  0.23 rad off the tracker's converged view; the guided round spawned toward the offset
  crosshair, and its chase crossed the window frames standing between the aim and the machine.
  The AT-420's own spread is ±0.003 rad [L record 0x2c +0x24 = 2.0 × 0.0014 — dump_08_at420] —
  not the source.
- The `walk00c`/`agent00_diag` runs' flight traces show it directly: the t747 launch's first dir
  was az 2.232 while the machine sat at az 2.031, the dir-stop shrank 2.9 → 0.9 u over 16 ticks,
  and the round died on the south-east corner post.
- The fix (d3-gated, the sisters' timelines byte-exact): the fight's tracker error, its slit and
  corridor gates, and the ready gate all read the CROSSHAIR's own bearing (the same law the shot
  flies), so a clamped cursor recenters before the shot; plus the eye-visibility gate (the eye's
  flight ray to the machine must reach t ≥ 0.8 — the wall-band-hidden launches read t 0.048
  [L the diag run]) and the machine-lane corridor from the eye and both gun stand-ins.
- **Measured result: 7 rockets, 7 body hits, damage 606 vs hp 598, best single rocket 101, no
  self-blasts, no world deaths** — the fight ends in ch 0x4b, the objective completes, the
  mission ends in the driving hand-off 0x0730008.

### 3. The done criterion

- `walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent`: **green,
  un-ignored** (the `#[ignore]` removed; the test doc carries the law). Real input only, no
  top-ups — the walk loads the level, plays the intro, the fusebox, the safe, the briefcase, the
  descent, the boarding, the ride, the cinematic, the shoot-out and the Hind fight, and ends in
  0x0730008, all on synthesized stick/trigger input with real damage.

## How it was verified (the binaries that ran, with their pass counts)

- `walkreal_07000008_live` whole: **6 passed, 0 failed, 0 ignored** (the Operative real-damage
  walk, the d2 Agent real-damage walk, the topped-up walk, both ally tests, and the 00agent
  sister).
- Every `walkreal_*_live` LEVEL binary that fit the window: **140 passed, 0 failed, 13 ignored
  across 19 binaries** — 01 (8/0/0), 02 (3/0/1), 03 (6/0/1), 04 (10/0/0), 05 (14/0/0), 06
  (6/0/1), 07 (5/0/0), 08 (6/0/0), 09 (8/0/0), 0a (12/0/3), 0b (7/0/0), 0d (1/0/1), 11 (3/0/1),
  12 (3/0/0), 13 (3/0/0), 14 (9/0/1), 15 (5/0/2), 16 (14/0/1), 1b (17/0/1). Nothing I changed
  touches the shared helpers (`tests/common/` is untouched), and every run is green.
- `nf-game` (lib + integration, `--test-threads=1`): **1047 passed, 0 failed, 8 ignored**.
- `nf-formats` + `nf-audio`: **136 passed, 3 failed, 0 ignored** — the 3 are exactly the slim-ISO
  `disc_golden` allowance (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).
- **The full four-crate suite did not fit this sandbox**: the sandbox's disk filled to 100 %
  while linking nf-view's ~100 test binaries (the linker died with `ld terminated with signal 7
  [Bus error]` — no scratch space left), and the window closed during the chunked re-runs. The
  binaries that did not run: the rest of nf-view's non-walkreal set, `walkreal_07000041_live`,
  `walkreal_0700004a_live`, `walkreal_0700004a_satchel_live`, the three `walkreal_0700000c_*`
  binaries, `walkreal_07000011_route_live`, `walkreal_celprobe_live`, `walkreal_dump_live`. The
  next session should run those (the builds are warm; the disk needs ~2 GB freed first — delete
  `target/release/deps` probe executables).
- The evidence probes (committed): `crates/nf-formats/examples/dump_08_panes.rs` (the 07000008
  pane timeline — the 0x20 Entity ops) and `crates/nf-formats/examples/dump_08_at420.rs` (the
  AT-420 record's flight words). The scratch probe `crates/nf-view/examples/probe_08_terminal_panes.rs`
  stays uncommitted (it says so in its header).

## What is left

- The unrun binaries listed above (disk-bound, not code-bound).
- The guard-side glass pass (the hind session's lead 2) — untouched this task, still documented
  in the hind report.
- Live checks for the main agent: none new — the shoot-out and the fight are asserted by the
  walk itself on real input. The one worth a PCSX2 glance: Bond fires the AT-420 from inside the
  bottom-terminal cabin at a LIVE window — the round should pass the glass (the window shatters
  from the mirrored contact) and fly on [M Collide_Update 0x1ea398:216-229] — the port's flight
  law the hind session landed, which this task's shoot-out depends on.

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).
