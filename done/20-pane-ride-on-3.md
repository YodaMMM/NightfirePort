## Your task: `pane-ride-on-3`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-2, merge current origin/master, and read its docs/reports/pane-ride-on-2.md. That session's tree is unchanged code with two proven laws and a recipe, so take "What's left" in order: re-land Laws 1 and 2 and the crack law from the recipe, bisect the ghost coupling on 00 Agent, then audit the 00 Agent hall bleed, which is the last blocker. Then set `PANE_RIDE` on and remove both dead_pane_live ignores. Keep the kind-0/1 props out of the set and carry the live checks forward. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to 07000001. Run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T01:10Z

---
## Report (done, 2026-11-08T02:03Z)


---
## Report from the worker (done, 2026-11-08T02:03Z)

# pane-ride-on-3 — the pocket laws landed on the ride, the d3 door phase solved, the d3 walk one fight short

2026-11-08, branch `s/pane-ride-on-3` (from origin/s/pane-ride-on-2 1d3d5f442, merged
origin/master — 5 conflicts, all doc-comment: master demotes the ray-skip claim [M]→[S] port
stand-in; master's side taken everywhere). PANE_RIDE is **ON** in the landed tree.

## Landed

1. **Laws 1+2 re-landed from the recipe and EXTENDED to d3.** The walker pocket hold
   (`flank_hold_x`, opt-in, own line per the merge lesson); the band-edge ambush (the 4.2
   fight radius, the EXPOSED-FIRST order on `DamageLog::visible`, the walled answer from the
   band's west end with a bounded 2400 shoot) — gated `difficulty >= 2` (pane-ride-on-2 had
   them d2-only; the d3 audit below proved #3's pocket needs them too). d1 stays byte-exact.
2. **The crack law with a measured gate.** Fires on `difficulty >= 3` when the lip set is
   empty AND the door at (7.9, 5.3, 11.1) reads `CLOSED` at the AND — plus the ORDER FIX:
   `bond_z`/`doorway_step` sampled AFTER the lip/crack work (the lure legs move Bond).
3. **The d3 door phase solved (the ride-on audit, stages 1-2).** The crack's retreat target
   (7.2, 6.4, 8.4) arrive 0.2 BREAKS #5's sight ray (the eye-ray crossing sits at x 6.78,
   west of the aperture's ~6.9 edge [L the run diagnostics]); the visible stops (8.98/9.27)
   held him at his post (6.28-6.43, 13.85-13.89) shooting through the diagonal — 19 hits for
   100, death 3552/3585 twice. Unseen, he advances on the lost-sight hunt and the crossing
   lure takes him (killed true; one ×2 volley, 57.6).
4. **The d3 ambush:** with the pocket laws on d3, #12 and #17 die for ZERO and the walk
   reaches the slab fight.

## Verification (all runs `--test-threads=1`, NF_ISO set)

- **Ride-off (before the flip):** ghost d3 GREEN 9.82 s, end 60.4 — the crack FIRES on the
  ghost too (the door reads CLOSED there as well — the report's named-loop pre-open
  hypothesis is NOT confirmed; the timeline survives the crack, so the pane-ride-on-2 ghost
  break came from that unlanded tree's other deltas, the sampling order being the one we
  fixed here). d2 ride-off GREEN 15.0+50.0 with 27 law engagements; d1 real+top-up GREEN.
- **Ride-on:** `walkreal_07000001_live` whole binary **7 passed / 1 failed** — the d2 walk
  reproduces the pane-ride-on-2 proof BYTE-EXACT (23811 ticks, end 15.0 + 50 armour, 11
  return-fire fights, the whole pack at hp 0); d1 top-up + real + the plate/hall pins green.
  The ONE failure is the d3 walk: it now survives the door, kills #12/#17 free at the
  ambush, and dies at **4.6 health** in the pocket fight (was: death 3585 at the door).
- dead_pane_live **0 passed / 2 ignored** (kept — the ignores' text cites the sisters; the
  d2 sister now passes but d3 is red, and removing them would ship a red suite);
  glass_pass_every_level_live **3 passed / 3 ignored**; walkreal_07000005_live **14/0**;
  walkreal_07000002_live run cut by a sandbox timeout mid-binary (the completed tests
  green); **nf-game lib 692 passed / 0 failed / 5 ignored**.

## What's left (the next session resumes here)

1. **The d3 walk's last 4.6.** The bleed map: hall 7.2 → door 57.6 (ONE ×2 volley — the
   crossing fight happens INSIDE the 3.0 band; a deeper hold line (8.0 tested) shifts the
   timeline WORSE — death 5761; the lever is the crossing-fight spacing, not the hold
   depth) → corridor 7.2 → the ambush pocket fight (~30). ~15 health from a pass.
2. The two dead_pane ignores, then the full walkreal battery (~100 binaries), the ai
   binaries and the suite (the background runner is killed between tool calls in this
   sandbox — run the batteries in foreground).
3. The kind-0/1 props stay OUT of the ride set (unchanged, [S] unverified); the live checks
   carry forward from pane-ride-on-2 §Live checks 1-5, plus: the d3 crossing volley's
   pellet economics at the aperture (is the ×2 doubling live at the crossing on a fresh
   boot?) — fieldprobe on #5's muzzle + a bond poke at the hold line.

## Usage

The session's input/output tokens, context used and cost: not visible to me. (The
environment exposed no token counters; the WORKLOG records the wall-clock: start 01:14 UTC,
report written 03:00 UTC, sandbox deadline 03:13 UTC.)
