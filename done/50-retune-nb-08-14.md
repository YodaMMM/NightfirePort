## Your task: `retune-nb-08-14`, 08 whole-mission 00 Agent real-damage walk green again (round 14)

Branch from current origin/master; if origin/s/retune-nb-08-13 is not on master, merge it in first. Read docs/reports/retune-nb-08-13.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-14`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-21T07:35Z

---
## Report (done, 2026-11-21T08:48Z)


---
## Report from the worker (done, 2026-11-21T08:49Z)

# retune-nb-08-14 — the 08 whole-mission 00 Agent real-damage walk, round 14

## What's done

- Branch `s/retune-nb-08-14` from `origin/master` (b69bacbc9) with `s/retune-nb-08-13` merged in
  (clean). Fresh sandbox: the repo, the game files (ISO 369823203-byte archive verified, work-slim)
  and the toolchain (rustup, the extracted `7zip`/`libasound2` debs, `cargo fetch`) were all set up
  this session. WORKLOG entry pushed as checkpoint 92c9899ab.
- **The baseline reproduces round 13 byte-exact** on the merged tree: death t29801 (frame 29803)
  at (113.08, 13.09, −179.88), 62 hits for 100.0, health 6.0 entering the killer salvo, the last
  world row before it f27904 — the crouched trench took ZERO rows from f27904 to f29803
  [L the 2026-11-21 baseline `NF_DMG_TRACE` run, 24.71 s].
- **Ledger lever (1) THE LAUNCH STARVATION — TRIED and DEAD on its own real run, reverted.**
  Implemented as the ledger described: the eye/lane stop threshold 0.7 and the height band 21.0
  for the WEST passes only (cp[0] <= 111.0), rockets >= 1 (stalemate-only, the dive launch's
  prefix byte-exact), the stance scheduler's stand gate widened to match. MEASURED
  [L the lever-1 run, 20.32 s]: a second launch DID fire at t1802 at the machine's west-high pass
  (53.07, 18.54, −174.05), dist 60.3 — but the death moved to t27583, **2218 ticks WORSE**:
  the west window kept Bond STANDING at the rim through the volume's burst rows (−2.00 × 10,
  t27509-27583, the no-crouch-while-window-open law) and the 60.3-u ride (~239 ticks at the
  0.2525 cruise) froze him mid-ride standing; the machine's reply kind-44 100.0 blast landed at
  (110.4, 14.4, −179.3), d 2.8. Measured inside the lever: the fired pass's own eye/gun ray stops
  were t0.862-0.875 ≥ 0.8 — **the 0.7 stop widening contributed nothing; the 21.0 band was the
  whole lever**, and the round-6/7 mid-band lesson (the ride freezes the dodge; the machine
  outruns the cruise round) repeats from the rim. The change was reverted (`git checkout`).
- **SHIPPED lever (2) THE TRENCH-PINNED STANDING DODGE (d3-gated, test-file only).** On a driving
  dodge (`salvo_close && (dodge_hold > 0 || salvo_live)`) the crouched Bond presses the stand
  (the shipped flip press, under the game's 60-tick substate lockout [M
  Player_ChangeSubState 0x190630:48]); `burst_hold` clears and the stance scheduler's fresh-zap
  arm is gated off while the dodge drives, so the hold cannot re-crouch him mid-salvo (the crouch
  returns when the salvo dies and a fresh volume row lands). Prefix-byte-exactness is measured,
  not assumed: all 112 w62 salvo rows in the baseline are the t3160-4030 crouched stalemate
  (zero standing rows), so `dodge_drive` never fired during a standing phase in the shipped
  timeline [L the baseline run]. The latch's both-open pick now scores the TRENCH side (the
  diagonal whose x-component walks the plate east wins — the round-13 lever-3's west walk was
  the band-corner death class), and a west-pointing chosen diagonal drops its forward half-press.
- **MEASURED [L the lever-2 run, 25.23 s]: the FIRST killer salvo is now SURVIVED.** The shipped
  −6.00 missile (dir (0.401, −0.162, 0.901)) was dodged standing: the miss grows
  0.58 → 3.24 → 2.41 (the crouched strafe held 0.56-0.58 the whole flight) and the falloff row
  drops −6.00 → −3.98 (health 6.0 → 2.0). The walk still dies — **t29924 (frame 29926) at
  (113.08, 13.47, −178.98), 123 ticks deeper than round 13** — to the SECOND missile
  (dir (−0.194, −0.109, 0.975), north-flying): the east-side diagonal walked Bond 1.4 u NORTH
  off the plate (z −179.0 → −177.65 at (112.54, 13.5, −177.6)) where the plate's east lane wedges
  the standing strafe (0.05 u per 30 ticks) and the walk-back south crawls ~0.016 u/tick; the
  dodge episodes ran with the miss STUCK at 1.40-1.41 for 40 ticks (the movement was along the
  missile's line, the east lane wedged) and its falloff −2.02 closed the 2.0-health walk.

## How it was verified

- Every change is inside `crates/nf-view/tests/walkreal_07000008_live.rs` — the test file only.
  No `crates/*/src/` change, no `crates/nf-view/tests/common/` change, so the affected-binary set
  is `walkreal_07000008_live` alone; the d1/d2 sisters never read the lever (the dodge stand
  press, the burst_hold clear, the arm gate and the trench pin are all inside
  `if difficulty >= 3` blocks).
- Each lever judged on its own real run of the d3 00-Agent walk with `NF_DMG_TRACE`:
  the baseline t29801 (24.71 s), the lever-1 west window t27583 (20.32 s, reverted), the lever-2
  standing dodge t29924 (25.23 s, shipped) [L three runs this window].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed (the target
  walk) in 176.48 s** — the d1/d2 real-damage sisters, the plain walks, the d2 Agent sister and
  both ally tests byte-exact green [L the final-tree run, `--include-ignored --test-threads=1`].
- The full suite (tools/test/suite.py): **did NOT fit the window, same environment as round 13 —
  background jobs are killed between tool calls in this sandbox** (verified twice this session:
  the nohup'd suite died twice between tool calls, suite.out emptied both times; the 10-minute
  foreground Bash cap also cannot hold the whole four-crate suite). What ran instead: the whole
  affected binary above (6/1, 176.48 s) plus the three lever runs. No `crates/*/src/` change this
  window, so every other binary is byte-identical in behaviour to the merged 08-13 state, whose
  numbers are in docs/reports/retune-nb-08-13.md.

## What's left (the next window's levers, in priority order)

1. **The second-salvo lane.** The first salvo's survival lane is the north-east DIAGONAL
   (the fwd 0.5 + the strafe, un-wedged, ~1.9 u) — so a z-band pin that FORBIDS the north walk
   would kill the survival. The answer must score, not forbid: score the diagonal on keeping z
   inside the pocket band −179.6..−178.9 while preferring the east x (its own run), or give the
   latch re-pick an interim south return leg before the episodes, or speed the walk-back's south
   return off the lip (the crawl is the lip's — the round-7 aimed-strafe crawl class).
2. The west-high launch starvation is MEASURED DEAD from the rim (the lever-1 run): a west launch
   needs the zap-free standing spot (the carried round-6 item) or the crouch-window discipline
   AROUND the ride, which the missile cam's input-zeroing forbids [M scene.rs ride block;
   Player_Update 0x193600 case 10].
3. The reply-cadence live check (carried from rounds 11-13).

## Live checks for the main agent

- The reply cadence (unchanged, rounds 11-13): a gdbwatch on the w62 spawn after the t49 dive
  launch — provoked or scheduled? It decides whether a delayed launch dodges the reply.
- The plate's east lane: from the standing capsule at (113.08, −178.96..−179.24), which triangle
  wedges the east strafe (the lever-2 run's 0.05 u per 30 ticks at x 113.08)? If the lane is a
  one-way lip (east closed, west open), the trench pin's scoring should read it from the world
  (a 2-u strafe cast, the same shape as the latch's 6-u diagonal cast) instead of the fixed
  east-preference.
- The walk-back south crawl (~0.016 u/tick on the lip): which surface stops the capsule at
  (113.08, −179.0..−177.7)?

## Usage

not visible to me
