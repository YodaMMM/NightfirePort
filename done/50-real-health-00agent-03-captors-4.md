## Your task: `real-health-00agent-03-captors-4`, 07000003's 00 Agent full-route real-damage walk goes green

Branch from current origin/master; if origin/s/real-health-00agent-03-captors-3 is not on master, merge it in first. Read docs/reports/real-health-00agent-03-captors-3.md there and the ignore message on walk_the_full_route_and_complete_every_objective_real_damage_00agent. Take next levers 1-2 from the ledger in the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000003_live binary and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-captors-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-14T07:54Z

---
## Report (done, 2026-11-14T10:04Z)


## Report (docs/reports/real-health-00agent-03-captors-4.md @ 76e4d602a on s/real-health-00agent-03-captors-4)

Task: continue the 07000003 00 Agent real-damage walk (ignored). The walk is NOT green; the ignore message on the branch (commit 76e4d602a) now carries this round's full ledger and the next levers. The walk stays ignored.

### What was done
- Branch from origin/master 385ebb66d, captors-3 (a283cb626) merged in first (not on master; clean).
- Runs 18-21 measured, real input only, helpers gated to 07000003 + d>=3: (18) the -3 ledger's lever 1 as written (resupply + switch_to_armed + re-take) REGRESSED to 10463 — the sprint walked Bond into the uncleared lane cluster, #10's 18-hit point-blank drain; (19) the same sprint cold survived the walk but the corridor's only pickup (14.2,-28.0) is an EMPTY gun (no ammo gain; attempt 1's (14.5,-29.0) sits behind the (14.0,-30.3) door) and died 10764; the lever pivoted to the kill-loot cycle; (20) run 20 (80288, the round's best, +5x over the -3 ledger's 15555-15776): the lane crossed at 41.2 with the re-armed clear engaging #6, the servants' phase reached, the end was the 0x23 plate's trio wake MID-mouth-nav — the si-1 saddle nav (from the spot the failed resupply dragged Bond to, the moat west end) detoured west and wedged 7x3000 on the lane doors, so the mouth nav rode the saddle zone (25.30,7.37,8.79) with the plate firing mid-climb; trio shotgun #5 charged to d 2.5 for 13.8 in 8, #7 chipped 27.4 in 43; (21) the saddle via the mouth-path anchor (22.1,-2.0): the anchor wedged 20x3000, the saddle nav then arrived (23.18,5.50,7.20) but Bond was DRY (the wheel cycled to the fists) and #7 landed 41.2 in 51 hip taps; died 74135.
- Wired (d>=3 only, the d<3 streams untouched): the lane-north kill-loot cycle + the si-1 saddle-wake block. The resupply-sprint shape is measured and removed.

### Verification (what ran, with pass counts)
- walkreal_07000003_live WHOLE on the lever tree: 6 passed / 0 failed / 2 ignored (305.66 s; the ignored = the d3 walk + the door-8 probe). The only later tree change is the #[ignore] string (no stream behaviour).
- interactables_07000003_live WHOLE: 9 passed / 0 failed (43.20 s).
- The FULL suite did NOT run: the background start died between tool calls (suite.out 0 bytes, no process survived); the foreground full build was killed by tool timeouts twice, then the sandbox disk filled twice (stale per-unit hash dirs under target/release/build/nf-view accumulated 4.8 G across killed invocations; target peaked 5.5 G on a 9.9 G disk). No test FAILED in any run; the suite's run phase never started. Diff scope: only walkreal_07000003_live.rs (+ docs), so the two named binaries + the direct d3 runs cover every test using the touched code.

### Live checks for the main agent
- #7 (skin 0x5000039, (11.4,10.1,-16.1) upper floor): do his boxes pass the eye-ray set from the saddle (23.5,6.3,8.4) (d ~26) or from the mouth (35.6,6.4,3.4)? The armed saddle stand (next lever 3) is his exposure test.
- Does the 0x23 plate fire when Bond crosses the saddle climb from the mouth-path side (run 20 measured it firing at (25.30,7.37,8.79)), and how fast is the trio's first volley after the wake (the sweep's settle+raise is ~330 ticks)?

### What's left
- The d3 walk (the refreshed ledger in the ignore message is the hand-off: arm the walk to the saddle — leg loot + the west-dragging resupply gated to d==2; the saddle route without the anchor; the #7 aimed stand; the trio wake's first volley).

### Usage
not visible to me
