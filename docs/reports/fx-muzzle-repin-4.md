# fx-muzzle-repin-4 — redo on the right base

Branch `s/fx-muzzle-repin-4`, cut from `origin/s/fx-muzzle-repin-2` (698c79b3f) as ordered, then
`origin/master` (d3865df75) merged on top (commit c6ef6d32d). The sandbox was reset, so the repo
was recloned and the setup (work-slim, ISO, rust, ALSA prefix) redone from the task message.

## The base check (quoted, as the task requires)

```
$ grep -c shot_flashes crates/nf-game/src/drone.rs
5
$ grep -c init_gt_shot_smoke crates/nf-game/src/gas.rs
4
```

Both above 0 on the merged tree — the muzzle code from s/fx-muzzle-repin-2 survived the master
merge. No game code was touched by this task; every draw (muzzle flashes, smoke, barks) stays.

## The master merge

Two test-file conflicts, both resolved to master's side (the newer landed doctrine):
- `crates/nf-view/tests/smg_close_range_live.rs`: master's audit-voice-triggers-4 rewrite of the
  pistol legs (the corridor-plate route, the retry/steal machinery) supersedes repin-2's older
  3-round tweak of the same legs.
- `crates/nf-view/tests/walkreal_07000003_live.rs`: both sides landed the identical lenient
  re-arm + moat-sidestep code; master's comments carry the newer run data.

## What's done, per binary

| binary | result | what changed |
| --- | --- | --- |
| `fix_npc_doors_live` | **GREEN 1 passed / 0 failed** (12.58 s) | nothing — the bark-escort fix and the muzzle draws coexist on the merged tree, first try |
| `walkreal_07000014_live` | **GREEN 9 passed / 0 failed / 1 ignored** (179.5 s; the ignore is the pre-existing 00 Agent wedge with its own documented reason) | one new route leg + one pin move (below) |
| `walkreal_07000011_live` | 2 passed / 1 failed / 1 ignored (the ignore is the pre-existing 00 Agent round-2 note) | the Operative real-damage walk fixed; the Agent real-damage sister still red (below) |

### walkreal_07000014_live (3 reds → green)

All three former failures (`the_e_use_switch...`, `walk_the_infiltration_to_the_delta_exit`,
`walk_the_tripbomb_route...`) wedged identically at "the corridor south to the leg-2 spot": the
shifted stream lands the post-landing drop east of the corridor lane ((30.52, 0.55, 34.13)
measured), the west leg arrives on its radius edge (30.48, 34.26), and the south leg's segment
hits the door wall's west jamb (cel 8 at 0.17 from (30.33, 0.55, 35.44)) — the walker then
E-opens the landing door whose swung leaf blocks the lane and the opener/auto-close loop eats
the 6000-tick cap. Fix: a jamb-lane leg `try_walk_to((29.4, 0.8, 36.7))` — stay west of the bar
until z is past it — added at both descent copies (the shared fn and the tripbomb's verbatim
duplicate). That flipped `..._real_damage` (previously green) red: the real-damage hostile
clear's lenient walk-home gave up 8 u short (z 30.75) and the post-clear position failed the
descent-landing assert. Fix: the landing assert now runs right after the leg-2 walk (it pins the
descent's geometry, which is what it says) and the hostile clear runs after it.

### walkreal_07000011_live (Operative real-damage fixed; Agent still red)

- d1 Operative: the 18.9-strip loot tour ratcheted Bond back UP the flights on the shifted
  stream (the |Δy| < 1.5 band pulled the fights' up-flight kill-drops one take at a time to
  y 24.01; the "reached the lower floor" assert fired at y 24.007). Fix: `loot_the_drops_floor`
  — the 18.9-strip call caps the band's up-side at the standing floor (a player descending
  scoops drops at his feet). Walk green: end health 30.0, 12 return-fire fights, all kills, the
  whole chain through 07000012 → 0700004A asserts pass.
- d2 Agent: died dry at the SE landing (tick 15551, (21.23, 40.27, 5.22), 37 hits for 100.0 from
  the 12-hp pair #13/#14 with weapon 0x19) — the dry body-block resupply toured to the placed
  grenade launcher #290 (10.54, 47.47, 18.10), wedged twice at the 6000 cap (desk cel 18), done
  (false), and Bond crossed the stairwell with no rounds. Fix that landed: the d2 pre-leg — the
  proven 00-Agent desk maze truncated at the mid corridor + the drop economy (the fights kill
  the spawn pair with clean positioning, the lane resupply scoops their drops: clip 4 spare 7 =
  11 rounds at the stairwell door vs the old run's 4). The dry death is GONE — the walk now
  survives the whole descent. Measured and retired: the launcher-box take never fires (the
  direct south hop wedges at (10.44, 15.81) d 2.29; the west pocket (9.6, 16.3) stands 2.03 out
  with the sphere line blocked) — and the Operative sister's own "clip 6 spare 6" resupply was a
  guard-drop fetch, not this box.
- d2 remaining red (one wedge from green): `walk_to the west flight A down (18.9→15.0)` —
  the #32 fight's backpedal (real-damage inside 3.0) dragged Bond to the ring's EAST side
  ((22.40, 23.99, −1.29), health 2.2) and the direct steer to the west flight cannot cross the
  central well (cel 128 at 0.076). Next lever (one leg, untested — the window closed): an
  18.9-strip re-anchor `try_walk_to((15.0, 19.2, 4.8))` before flight A, or a nav-routed
  re-anchor around the ring.

## Verification

Every fix is a walked re-pin on the merged tree (no teleports, real input, no game-code edits,
no draws removed or reordered). Runs this session, each binary whole, `--test-threads=1`:

- `fix_npc_doors_live` — 1 passed / 0 failed.
- `walkreal_07000014_live` — 9 passed / 0 failed / 1 ignored (pre-existing ignore).
- `walkreal_07000011_live` — 2 passed / 1 failed / 1 ignored (pre-existing ignore; the Agent
  sister is the failure).
- `walkreal_07000003_live`, `smg_close_range_live` — compiled on the merged tree (their conflict
  resolution took master's landed pins); NOT run — the sandbox window went to the three task
  binaries.
- The full suite did not fit the sandbox window (fresh sandbox: setup + cold build consumed the
  first hour). Rule-10 grep of the touched test files: the edits are confined to
  `walkreal_07000011_live.rs` and `walkreal_07000014_live.rs` (both run whole above); no shared
  helper under `crates/nf-view/tests/common/` was changed, so no other binary can shift from
  this branch's diffs except through the two conflicted files noted above.

## What's left

1. The d2 Agent re-anchor leg (above), then rerun `walkreal_07000011_live` whole.
2. `walkreal_07000003_live` + `smg_close_range_live` runs on the merged tree, then the full
   suite (slim ISO: only the 3 `disc_golden` tests may fail).

## Live checks for the main agent

- `tools/trace/fieldprobe.py` on 07000014 door placement (32.0, 2.1, 35.8) (kind 2): poke it
  open (state 0x4, pos01 1.0) and screenshot from (30.3, 0.6, 35.4) facing south — expected: the
  swung leaf's west edge reaches x ≈ 30.3 and blocks the corridor lane, which is the wedge the
  jamb-lane leg routes around.
- `tools/trace/gdbwatch.py` on the pickup take check [M 0x18e148:203-214] while Bond stands at
  (9.6, 16.3) facing the launcher box (10.54, 47.47, 18.10) on 07000011: expected the sphere
  line test to fail (blocked) — confirming the retired west-pocket approach cannot take #290.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).
