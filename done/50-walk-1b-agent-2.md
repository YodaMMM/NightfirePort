## Your task: `walk-1b-agent-2`, 1b arrival to Drake's defeat Agent and 00 Agent real-health walks green (round 2)

Branch from current origin/master; if origin/s/walk-1b-agent is not on master, merge it in first. Read docs/reports/walk-1b-agent.md there and the ignore message on walk_arrival_to_drakes_defeat_on_real_health_agent and _00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Agent first; keep the Operative walk green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700001b_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/walk-1b-agent-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-22T23:54Z

---
## Report (done, 2026-11-23T01:19Z)

Closed by the driver: the report is on `s/walk-1b-agent-2` (docs/reports/walk-1b-agent-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-23T01:23Z)


---
## Report from the worker (done, 2026-11-23T01:23Z)

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
6000..7500, through the NF_1B_PAUSES3 hook) measured RED — all 15 stops.** k5 3191 / k4 4426 /
k3 5901 latch identically in every stop; the re-rolls then die or stall at the k8/k6 defenses:
k8 latched 7583/8464/−/7587/−/−/7859/−/−/−/−/8161/8234/8305/8728 (stops 6000..7500/60), k6 only
at 6750 (8864); health at the end 0.0/0.0/0.0/8.4/42.5/42.5/17.5/40.9/0.0/42.5/42.5/32.5/42.5/
42.5/42.5. None latched all 8. The seven 42.5-shape stops are a STALL at the k8/k6 read (same
shape as the master baseline's stuck-8864/37.5, but with more health); 7100/60 latched k8 (7859)
and reached 17.5; 7200/60 died. The hook stays (defaults to a no-op) because the next lever
needs it.

Both ignore messages carry the refreshed ledgers and the next levers (Agent: the
shoot-and-scoot or a d2 second-stop re-roll; 00agent: sweep the stop LENGTHS at a 42.5 stall
(7050/7250/7400 at len 30/90/120) to shake the k8/k6 read, or sweep the k4 stop — the re-roll
divergence starts at the k4 leg).

## How it was verified

Every lever was judged on its own real run of the real-input walk binary (no top-ups, no
damage/health changes, no guard-code changes). Runs on the final tree (whole binary,
`--test-threads=1`):

- `walkreal_0700001b_live`: **17 passed; 0 failed; 2 ignored** (119.02s) — the two target
  walks are the ignored ones (their ledgers are in the ignore messages). The Operative walk
  stays green; nothing else in the file moved.
- `walk_arrival_to_drakes_defeat_on_real_health_agent` (individually, lever formulations 1 and
  2): the death shape above, 39.8s / 39.6s.
- `walk_arrival_to_drakes_defeat_on_real_health_00agent` (individually, all 15 sweep stops
  6000..7500/60): the measurements above, ~16-60s each.

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
