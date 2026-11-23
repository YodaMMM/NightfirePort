# walk-1b-agent-2 — 1b Agent / 00 Agent real-health walks, round 2

Branch `s/walk-1b-agent-2` (origin/master 7562f1e70, with origin/s/walk-1b-agent merged in
first — it was 5 commits ahead of master: merge commit 22cd90615). The sandbox was reset (no
repo, no rust, no game files), so setup ran from scratch: clone, work-slim + ISO downloads,
rustup, ALSA user prefix, queue worktree.

## What's done

**Agent walk (`walk_arrival_to_drakes_defeat_on_real_health_agent`) — the round's named lever
(the Drake-fight aim/line, the r11 open lever) measured RED, reverted to the round-22 shape.**
Two formulations, each judged on its own real run of the real-input walk:

1. **The Drake-first aim pick** (`drake_first` in `fight_opts_y`, gated to this level + the
   Agent walk): every 10 ticks, when Drake's `exposed_aim` is Some (a clear eye→box bullet
   segment — the same rule the port's bullet pass runs [M 0x1ea398:216-229]) and no wave
   shooter sits in the 3.5 u double-damage band [M DoBulletAccuracy 0x179740:113-115], the aim
   pick overrides to Drake so the tap's level ray rests on his box. Measured: `exposed_aim(Drake)`
   NEVER passed — the west shelter's sight-cut cuts his box line at his ~60 u spawn — so the
   fight kept tapping the wave: 7 taps (clip 100→16), wave 16→13, **Drake 120→120**, Bond
   23.7→13.2 in the first 480 ticks (3 × 3.5 astronaut hits, zero Drake hits by f13200), dead
   inside the fight (~f13700) with Drake still ~50-52 u out.
2. **The weapon-range fire probe**: the drake fire gate probes the flight's own death range
   `weapon_data+0x1C` [M `Bullet_update` 0x12e7b0 case 1] instead of the 40 u pick cap.
   Bit-exact identical run — the line never opened either way.

Facts banked for the next round ([L] this branch's runs): Drake closes ~0.018 u/tick
(x −7.3 → 1.2 in 479 ticks); the aim lift out of the ZERO-LOSS waves-wait shelter itself costs
the wave's sight (the lift is what exposes Bond — the shelter is zero-loss only while the eye
stays in the cover band). The gate ships OFF (`false` at the call site — the shipped shape is
byte-identical to round 22); the scaffolding stays for the next round's shoot-and-scoot.

**00 Agent walk (`..._00agent`) — the round's named lever (the 60-tick mid-phase stop sweep
6000..7500, through the NF_1B_PAUSES3 hook) measured RED at the five sampled stops.** Stops
6000/60, 6250/60, 6500/60, 6750/60, 7000/60: every re-roll dies mid-phase BEFORE the shared
k8/k6 read — consoles latched 4/4/3/5/3 (k5 3191, k4 4426, k3 5901 everywhere; k8
7583/8464/−/7587/−; k6 only at 6750: 8864), health 0.0/0.0/0.0/8.4/42.5 at the end. None
reached the old stuck-8864 depth; 7000/60 was the least bad (hp 42.5, and the only stop that
latched k6). The hook stays (defaults to a no-op) because the next lever needs it.

Both ignore messages carry the refreshed ledgers and the next levers (Agent: the
shoot-and-scoot or a d2 second-stop re-roll; 00agent: finish the sweep 7050..7500/60 and the
length sweep at 7000, or sweep the k4 stop).

## How it was verified

Every lever was judged on its own real run of the real-input walk binary (no top-ups, no
damage/health changes, no guard-code changes). Runs on the final tree (whole binary,
`--test-threads=1`):

- `walkreal_0700001b_live`: **17 passed; 0 failed; 2 ignored** (119.02s) — the two target
  walks are the ignored ones (their ledgers are in the ignore messages). The Operative walk
  stays green; nothing else in the file moved.
- `walk_arrival_to_drakes_defeat_on_real_health_agent` (individually, lever formulations 1 and
  2): the death shape above, 39.8s / 39.6s.
- `walk_arrival_to_drakes_defeat_on_real_health_00agent` (individually, 5 sweep stops):
  the measurements above, ~16-60s each.

No `src/` change and no shared-helper change (everything is inside
`crates/nf-view/tests/walkreal_0700001b_live.rs`). `ray_on` now wraps a probe-length
`ray_on_len` with the same 40 u default — every other caller is identical, so the d1/d3 routes
stay byte-identical; the d1 walk's green run above confirms it.

## What was run, and what was not

- Ran: the whole `walkreal_0700001b_live` binary on the final tree (17/0/2 above) plus the two
  target tests individually after every lever;
  `cargo test --offline --release --no-run -p nf-formats -p nf-audio -p nf-game -p nf-view`
  builds all four crates clean.
- Did NOT run: the fast-suite runner (`tools/test/suite.py`). This sandbox kills background
  jobs between tool calls (verified with a `sleep` probe), so the prescribed `nohup` start
  produced nothing; the foreground window went to setup (clone, downloads, rustup, and a
  full-disk incident — the nf-view build scripts had duplicated 1320 lib rlibs for ~5 GB,
  cleaned and rebuilt) plus the lever runs. Same justification as round 22: the change surface
  is the single 1b test file, and the whole 1b binary ran twice on the final tree.

## What's left

- **Agent green**: the Drake-fight kill. The named lever for the next round is the
  shoot-and-scoot out of the west shelter (sit the ZERO-LOSS shelter, leave it only on a
  Drake-line window — probe a ring spot whose muzzle ray to Drake is clear, scoot, tap,
  return; the lift costs 3.5/hit ×3 per 480 ticks at the shelter mouth, so the scoot must be
  brief) — or a d2 second-stop re-roll (the d2 stream has never been re-rolled by a second
  stop). The scaffolding (`drake_first` in `fight_opts_y`, the range-aware `ray_on_len`) is
  already in place, gated off.
- **00 Agent green**: finish the mid-phase sweep (7050..7500/60 and lengths 7000:30/90/120 —
  the 7000 window re-rolled the k8 latch to 7587 and latched k6 for the only time), or sweep
  the k4 stop (4426 vs the d2 walk's 4404 — the re-roll divergence starts at the k4 leg).
  The NF_1B_PAUSES3 hook makes all of these one-line runs.

## Usage

not visible to me.
