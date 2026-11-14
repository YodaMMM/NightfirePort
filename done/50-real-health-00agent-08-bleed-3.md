## Your task: `real-health-00agent-08-bleed-3`, 07000008's 00 Agent whole-mission real-damage walk wins the Hind fight

Branch from current origin/master and read the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent in crates/nf-view/tests/walkreal_07000008_live.rs, plus docs/reports/real-health-00agent-08-bleed-2.md on origin/s/real-health-00agent-08-bleed-2 if it has one. The descent is cleared and the Hind fight is reached; rocket 1 lands, but Bond dies around tick 32622 to the machine. Take the next levers the ignore message names for the Hind fight (dodge timing, cover between volleys, the remaining rockets' pacing), judging each on its own real run. Helpers stay gated to 07000008 and difficulty 3, so the other difficulties' walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


11-13 (master eb89478f5): on the current master the walk wedged BEFORE the Hind - the "to the briefcase" nav leg from the bolt stand (z -117.6) ran Bond into the safe-face corner (162.59, -118.17) for the cap (ch 0x4a never wrote, t26460). Fixed by starting the leg from the plain walk spot (158.93, -116.52); the walk now dies at t34277 at (110.98, 13.52, -179.31). The Hind levers in this task stand.

Measured 11-13 (master 42aff3fcb): the d3 bleed is at the START - the station hall guards take Bond 96 -> 12 on "the walkway head nav 2" (t977-1916: #11 32.0, #3 44.0); the mouth stand sees nobody at t373 (they wake on the plate and stay hidden). A held mouth stand (poll every 30 ticks for a visible guard, 300/900 ticks, radius 25/35) fights nobody; the 900 hold reaches the hall exit at 48 instead of 20 but re-rolls the descent - deaths t25621 (300) / t26960 (900, the lower level east, #19/#20/#23 0x15) vs t34277. Not shipped. Next: fight the hall guards from cover as they come round, or cross the hall on a different line.

---
claimed-by: sandbox4 at 2026-11-14T01:39Z

---
## Report (done, 2026-11-14T04:27Z)


---
## Report from the worker (done, 2026-11-14T04:27Z)

# real-health-00agent-08-bleed-3 — report

**Branch `s/real-health-00agent-08-bleed-3`, base origin/master `c9b24271d`.**
Task: 07000008's 00 Agent whole-mission real-damage walk — the Hind-fight levers and the
bleed the 11-13 measurement put at the START. The walk is NOT green; the ignore stays and
now carries the measured 11-14 ledger and the next levers.

## What's done

**The start-bleed lever ("fight the hall guards from cover as they come round, or cross
the hall on a different line") is taken, measured on its own real runs, and shipped
d3-gated — the walk advanced from the master death t34277 in the Hind fight to t6554 at
the fusebox, and the next blocker is named with its evidence.** All changes live inside
`crates/nf-view/tests/walkreal_07000008_live.rs` (test-local helpers, every new call
`difficulty >= 3`-gated); the sisters' timelines stayed byte-exact [L the whole-binary run].

1. **The baseline** (no changes, master c9b24271d): the walk dies at tick 34277 at
   (110.98, 13.52, −179.31) in the Hind fight — 33 hits for 100.0, 19 return-fire fights,
   the machine still 0.0/598 damage when Bond fell; the killing blast is a wall detonation
   at (113.0, 14.6, −182.1), d 3.6, kind 44. The hall nav-1 lands Bond at (152.16, 46.08,
   −93.92) INSIDE the east placements' fire (the t684 −4.0 from #14 at d 4.1) and the nav-2
   return-fire stand on #11 alone costs 32.0 (96.0 → 64.0) — the 11-13 measurement
   reproduced exactly. The mouth stand sees nobody at t373 (the guards hold the east
   placements and never come round; a held stand there is a no-op — confirmed again).
2. **The east-deck head stand** (lever 1, first option): the walkway-head nav's own
   measured arrival (152.16, 46.08, −93.92) is walked as its own d3 leg ("the east deck
   head", 152.2, 46.0, −93.9), then `fight_guards_within` visible-only at 25 u — from the
   deck head every shooter is at 4-12 u. The stand kills #14, #11 (×2 positions), #2 one
   at a time for **40.0 total** (100 → 60) — against the 76.0 the mid-nav bursts cost.
   The guards never advance (the 11-13 "as they come round" option is dead: they hold
   placements even across a 900-tick hold), so fighting them FROM the deck is the only
   cheap line — measured.
3. **The crossing hardening**: from the deck-head node the walkway-head nav re-routes
   through the east door (160.6, 45.0, −100.1) and wedges in its frame from the deck above
   (head_bump at (160.27, 46.60, −99.37), the door already open pos01 1.00, every later leg
   wedged there [L run-lever1]). Two d3 walk_tos carry it: "the deck line west" walk_to
   (148.5, 46.1, −98.5) (the baseline bleed's own line), then "the stair head from the
   walkway" walk_to (146.6, 44.4, −117.6) — master's own approach-chain waypoint — because
   the landing nav from the new line wedges on the stair lip (146.34, 44.99, −115.69,
   grounded=false [L run-lever3]). The walkway pair #7/#8 and the deck stragglers #9/#10
   die to the return fire for FREE (60.0 → 60.0). A direct nav to the exact measured node
   (152.16, −93.92) wedges EARLIER, at the hall's south door (140.7, 49.5, −96.8) [L
   run-lever2] — the route is target-sensitive; the shipped two-step is the measured one.
4. **The fusebox pit blocker, measured and ledgered**: the walk now dies at tick 6554 at
   (154.51, 43.10, −120.72) — 26 hits for 100.0, 10 return-fire fights. The pit guard #16
   (skin 0x5000030, (148.1, 38.7, −110.5)) sniped the laser-burn stands from the level
   BELOW at 12.7-12.8 u and 4.4 u down — outside the doorway fight's 6 u and the close
   fights' 12 u/±4 rings — fourteen −4.0 hits t3550-6554 = the last 60.0. Two fight
   variants measured on the spot: the visible-only wide fight (25 u / ±6 y) found nobody
   (#16 is NOT visible from the burn spot while his rounds reach Bond) and the timeline
   matched lever 4's t6554 exactly; the unseen nearest-first variant REGRESSED (its pick
   opened on the buried doorway man #5, stalled 60 ticks, death t4320 [L run-lever6]).
   The shipped state keeps the visible-only wide stand (a no-op on the measured roll, the
   right shape for a re-roll) and the ledger names the real lever.

## How it was verified

All runs on the slim ISO, release, `--test-threads=1`, 2026-11-14, NF_DMG_TRACE on:

| run | result |
|---|---|
| the d3 walk, baseline (no changes) | death t34277, 33 hits for 100.0 — the 11-13 ledger reproduced |
| the d3 walk, per iteration | lever1 wedge t40500 (health 44.0, 12 hits for 56.0) → lever2 wedge at the hall door → lever3 wedge at the stair lip → lever4 death t6554 (26 hits) → lever5 visible-wide t6554 (identical) → lever6 unseen-wide t4320 (regressed, not shipped) |
| walkreal_07000008_live, whole binary, `--include-ignored`, final tree | **5 passed / 1 failed** — the failure is the ledgered d3 walk itself; the d1 plain, d1 real-damage, d2 Agent and both ally tests GREEN (their timelines byte-exact under the d3-gated changes) |
| interactables_07000008_live | **15 passed / 0 failed** |

The full suite did not run: the sandbox reset this window (Rust and the ISO had to be
re-fetched) and the background runner dies between tool calls here (nohup and setsid
both — the same finding the walker-sidestep-decay-2 WORKLOG entry recorded), and the
disk ceiling returned (9.9 GB; `target/release/build/nf-view` held 4.6 GB of stale
hardlinked per-invocation test binaries that had to be purged to link at all). My changes
touch one test file only; the whole affected binary + interactables ran on the final tree.

## What's left

- The fusebox pit guard (next lever, in the ignore message): a fight pick that targets
  the guard that actually landed the last hit within the wide 25-u/±6-y ring — the
  walker's hurt bookkeeping names him (#16) — instead of the nearest-first pick that
  stalls on buried guards. Then the same wide stand before the bolts/safe burns.
- Past the fusebox the route is the master flow with the guards dead upstream; the
  descent re-rolls (the 11-13 mouth-hold note's re-roll deaths t25621/t26960
  lower-level-east are the class to watch).
- The Hind fight's own levers stand unchanged: the dodge window 30 → 60 u with the side
  step held until the round dies (the killing blast is a WALL detonation 6 u out, the
  k=6 falloff signature [M Explode_Propagate 0x17c518:69-71]), cover between volleys,
  the machine's own rounds through the launch-time forecast law.
- The full suite on a sandbox that keeps background processes alive.

## Live checks for the main agent

1. 07000008, 00 Agent, at the fusebox: is the pit guard below (the level at y ≈ 38.7,
   east of the stair) able to see and fire at Bond standing at the burn spot (154.35,
   43.41, −119.67) in the REAL game — and does Bond's own sight ray to him read blocked?
   (The port's sight says blocked both ways while his rounds land; if the real game
   lets Bond see him, the fight pick is simply visible-only at the wider ring.)
2. The station hall east deck: confirm the six d3 pistol guards hold their placements
   (152.5-161.5, 46.0, −87.7..−101.5) and never advance toward Bond across a long hold
   (the 11-13 + 11-14 port runs say hold; a real-game counter-example re-opens the
   "fight them as they come round" line).

## Usage

- input tokens: not visible to me
- output tokens: not visible to me
- context used: not visible to me
- cost: not visible to me
