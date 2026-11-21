## Your task: `retune-nb-08-15`, 08 whole-mission 00 Agent real-damage walk green again (round 15)

Branch from current origin/master; if origin/s/retune-nb-08-14 is not on master, merge it in first. Read docs/reports/retune-nb-08-14.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-15`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-21T13:30Z

---
## Report (done, 2026-11-21T14:15Z)

Closed by the driver: the report is on `s/retune-nb-08-15` (docs/reports/retune-nb-08-15.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-21T14:24Z)

# retune-nb-08-15 — the 08 whole-mission 00 Agent real-damage walk, round 15

## What's done

- Branch `s/retune-nb-08-15` from `origin/master` (ae5fc9219) with `s/retune-nb-08-14` merged
  in. Fresh sandbox: the repo, the game files (work-slim, the 369823203-byte ISO archive
  verified) and the toolchain (rustup 1.100, the extracted `libasound2`/`libasound2-dev` debs,
  `cargo fetch`) were all set up this session. A stale remote `s/retune-nb-08-15` (an earlier
  session's setup-only branch, two commits, based on a master 16 commits older) was superseded
  by a force-push of this branch — it contained no lever work and no report, and its merge
  resolution of the walk's ignore line was identical to this one. The merge's only conflict was
  the walk's ignore line; it kept the 08-14 ledger (master's run-12 census finding is the
  ledger's round-5 lockout answer — the same decompile addresses, the same t20944 death —
  already recorded in the ledger).
- **The baseline reproduces round 14 byte-exact** on the merged tree: death t29924 (frame
  29926) at (113.08,13.47,−178.98) standing, 63 hits for 100.0 [L the baseline `NF_DMG_TRACE`
  run, 24.66 s]. New measurement: the killer-salvo chain is **8 w62 rounds t3160−t4150
  (frames 28926−29916) from 8 rotating bearings, ~55−100 ticks apart**; the first ~6 pass
  CLEAN, and the walk closes on two rows only.
- **Ledger lever (1) THE SECOND-SALVO LANE, variant (1a) the band-scored latch pick — TRIED
  and DEAD on its own real run, reverted.** The pick scored each candidate diagonal by the
  travel distance until its z left the pocket band −179.6..−178.9 (along-band diagonals
  scored the 3.0-u horizon cap; the east preference stayed as the tie-break; the open-side
  clearance order unchanged). MEASURED [L the lever-1 run]: the north-wedge row is GONE (the
  north-east round passed miss 3.41 → 4.08 clean) but the flipped pick walked the exits
  SOUTH — −1.34 at the south corner (113.0,−180.8), then the chain's last round re-aimed
  through the south-shifted chest (dir (0.065,−0.158,0.985)) detonated at (112.9,−182.2) d 3.2
  for −4.66 — the same −6.0 total drain, death t29917, 7 ticks WORSE. The chain's detonation
  ring sits 2.5−3.5 u out on several bearings around the 0.7-u pocket: two rows of
  −1.3..−4.7 are unavoidable FROM the pocket, so the chain is un-survivable at the 6.0 entry
  health and survivable at ~10−12.
- **The upstream stalemate cannon dodge — TRIED in two steps, both DEAD, reverted.** Step 1:
  the w61 gate's `rockets < 3` exclusion narrowed to "only while the machine is inside the
  west launch band" (cp[1] ≤ 13.7 && cp[0] ≤ 111.0 keeps the exclusion; outside it the cannon
  rounds may start a dodge). MEASURED [L the lever-2 run]: the hold latched (dh 21−24 on 156
  sampled w61 rows) but the shipped d3 drive is w62-only (`salvo_close && (dodge_hold > 0 ||
  salvo_live)`), so no input was set — the damage timeline stayed byte-identical, death t29924
  unchanged. Step 2 added the stalemate drive term (`dodge_hold > 0 && !salvo_close &&
  w61_close`) with the stand press and the `burst_hold` clear gated to `salvo_close` (the
  crouch holds — the zap-free trench) and the stalemate episodes strafe-only anchor-ward.
  MEASURED [L the lever-2b run]: 58 hits (5 chips dodged) BUT the crouched strafe
  (0.02 u/tick, latched through most of each ~30-tick pass) drifted Bond out of the pocket
  onto the south-west rim — (112.85,13.47,−180.87) STANDING — and the walk died t27885,
  **2039 ticks WORSE**, the round-13 lever-3 west-walk death class. Both steps reverted.
- The shipped round-14 state is the final tree. All changes this round lived in
  `crates/nf-view/tests/walkreal_07000008_live.rs` (the test file only) plus the WORKLOG and
  this report; no `crates/*/src/` and no `crates/nf-view/tests/common/` change, so the
  affected-binary set is `walkreal_07000008_live` alone.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk with `NF_DMG_TRACE`: the
  baseline t29924 (24.66 s), the lever-1 band-scored pick t29917 (reverted), the lever-2
  stalemate gate t29924 byte-identical (reverted), the lever-2b stalemate drive t27885
  (reverted) [L four runs this window]. The prefix through the dive window stayed byte-exact
  in every run (the [08 rocket] t49 launch — 101.0 landed — printed identically each time).
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed (the
  target walk) in 177.54 s** (`--include-ignored --test-threads=1`) — the d1/d2 real-damage
  sisters, the plain walks, the d2 Agent sister and both ally tests byte-exact green, matching
  the round-14 final-tree numbers [L the final-tree run].
- The full suite (tools/test/suite.py): **did NOT fit, same environment as rounds 13−14.** The
  background fast runner was started in the first minutes and was killed between tool calls
  (suite.out 0 bytes, the process gone — the same kill the 08-14 report verified twice). The
  10-minute foreground Bash cap cannot hold the four-crate build+run (the 08-14 report's
  measurement), and no `crates/*/src/` change was made this window, so every other test
  binary is byte-identical in behaviour to the merged 08-14 state, whose numbers are in
  docs/reports/retune-nb-08-14.md. The affected binary was run whole (above).

## What's left (the next window's levers, in priority order)

1. **The walk-back family (untried):** (1b) give the latch re-pick an interim south return
   leg before the episodes — a re-pick from a north-of-band spot first presses into the band,
   then runs the episodes; or (1c) speed the walk-back's south return off the lip — with the
   lever-2b drift data: any drive-term variant must cap the per-pass drift (e.g. dodge_hold
   capped at ~8 ticks for w61-only episodes) so the walk-back out-runs the drift between
   passes; the uncapped strafe walked Bond onto the rim.
2. The sustained-burst 3-row cost (f26198−26211, −6.0): the crouch plan's flip takes ~3 rows
   under the 60-tick substate lockout [M Player_ChangeSubState 0x190630:48] — whether the
   crouch press can flip faster, or the plan can pre-crouch on the volume's ARMING frame
   (~f25666) instead of the first row, is a live-check question.
3. The trio 36.0 and the reply-cadence live checks (carried from rounds 8−14 — every
   deck-leg code lever measured dead).

## Live checks for the main agent

- The reply cadence (carried, rounds 11−14): a gdbwatch on the w62 spawn after the t49 dive
  launch — provoked or scheduled? It decides whether a delayed launch dodges the reply.
- The chain's detonation ring: the 8 w62 rounds' detonation points measured this round —
  (110.3,16.5,−179.0), (111.2,13.4,−176.7), (112.9,−182.2) — sit 2.5−3.5 u from the pocket.
  A fieldprobe sweep of the terminal's north lip (z −176..−178, x 111−113) would settle
  which triangles catch the north-flying rounds, and whether any standing spot outside the
  pocket clears the ring (the carried round-6 "no standing spot on the lower floor" law
  bounds this from below).
- The sustained burst's 3-row flip latency (item 2 above): trace BL+0x960 through
  f26196−f26215 to see whether the lockout (not the plan) eats the first press.

## Usage

not visible to me


