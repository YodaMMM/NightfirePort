## Your task: `los-mask-repins-02-2`, 07000002 can be walked with every objective on 00 Agent without the civilian being shot

Branch from origin/s/los-mask-repins-02, merge current origin/master, and read its docs/reports/los-mask-repins-02.md, "The traced cause" and "What is left (the fix)". The d3 castle walk fails because west-end rifle guards #10 and #11 (placements 452/453) shoot the scared civilian #22 while Bond crosses the upper-east stretch. First settle item 1: are placements 452/453 d3-only (Drone_Create's minimum-difficulty gate [M 0x136be0:49]), or alerted late on the d3 timeline? Read the placement bytes [D]. Then take its levers in order: beat the alert, a lane-side clear before the descent (probe the route first), then the silenced west-wing kills. The stair-top stand is a measured dead end. Helpers stay gated to 07000002 and the difficulty, so other walks stay byte-identical. Remove the temporary diagnostics before the final commit. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000002_live binary and interactables_07000002_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-repins-02-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T03:56Z

---
## Report (done, 2026-11-11T04:40Z)

# los-mask-repins-02-2 — report (2026-11-11)

Branch `s/los-mask-repins-02-2`, cut from `origin/s/los-mask-repins-02` (e73606e57), merged
current `origin/master` (clean; the base's report and its two live checks were read first).
**Status: DONE — the 07000002 00 Agent real-health walk is green and un-ignored; the scared
civilian survives the whole route.**

## What's done

**Item 1 (settled first, from the disc): placements 452/453 are NOT d3-only.** The level's
main map carries exactly 25 drone (0x0f) placements, #442-#466 consecutive, and every one has
**param 2 = 1** [D census dump, `census_07000002_drones`] — `Drone_Create`'s minimum-difficulty
gate [M 0x136be0:49: spawn iff param 2 ≤ difficulty] passes them at every difficulty. The walk's
drone numbering maps 1:1 (drone #10 = placement 452, #11 = 453, #22 = the civilian 464 —
confirmed by the run's per-drone placement prints). The d1/d2 walks pass because of **alert
timing**, not a roster difference.

**The traced alert chain [L the per-leg state trace this round]:**

- Both west-end rifles sit idle (state 1) at their start posts (#452 (−77.6, −7.3) far west,
  #453 (−42.5, −31.5) south-east) through the plate leg (t5586).
- The geisha-1 duel's LOUD shots turn them within ~200 ticks (state 7 = PATROL, t6320); #10
  converges 23 u east to the west-end post (−52.1, −1.6) in COMBAT_NO_ROUTE 0x5F by t7942; by
  the upper-east crossing (t12200+) both sit on sight lines that pass **0.6-1.0 u from the
  civilian's hide spot** (inside the ±1.5-u miss wobble [M DroneWeap_DoBulletAccuracy
  0x179740:136-148]) — #11's miss round killed her at t12239, the ONLY guard→drone round hit of
  the walk [L the stair-bottom diag: `round hit t12239: #11 → #22`, her hit (weapon 0x12,
  part 5, damage 1.5), ch 0x60 ON → the mission fail].
- The d2 comparison run: the same duel turns them (state 7 at t6404) but its guards end the
  crossing FAR WEST investigating (#11 GoToGoalPosition 0x63 at (−82, −19), line 2.9 u clear;
  #10 in DEATH_ANIM — already dead) — the d2 safety was positional luck on a faster timeline.

**Lever (b) probed first, measured dead:** the wing-interior clear does not exist — #9/#10/
#11/#12 sit behind walls from every wing eye (the armour spot, the wing hall, the wing east,
the wing door; `segment_bullet` wall ×20) [L the probe at t10894]. The stair-top stand is the
base's measured dead end and was not repeated.

**Lever (c) IS the fix: the silenced geisha-1 duel.** The P99's fire sound **0x403 has outer
radius 40** [D docs/systems/weapon-fire-sound-ids.md + sound.md: 0x403 inner 2 / outer 40 /
weight 65], `Sound_Alertness` cuts sounds past the outer radius, and `DroneVision_AlertSound`'s
share > 0.2 gate never opens [M 0x177800] — while the guards' start posts sit **41-48 u** from
the duel area. The d3-only arms (all inside `walkreal_07000002_live.rs`, gated on the
difficulty; the d1/d2 paths byte-identical):

1. `before the geisha-1 room` arms the P99 instead of the fullest 0x12 (d3 only).
2. The round-0 stand-off loop no longer arms the dropped shotgun per kill (d3); the scoops stay.
3. `clear_captors_real` gained a `quiet` flag (this file only): the geisha-1 room fight keeps
   the P99 in hand instead of `switch_to_armed`'s loudest-deep-gun pick.
4. The measured-and-reverted variants: the 900-tick hold + the 8.0 approach + the 6.0 room ring
   all made the duel bleed WORSE (Bond dead mid-phase at t14094 — standing in the closing
   captain's volley ring); the 240 hold + the 3.5 approach + the 4.0 ring stand.
5. The duel still costs Bond ~97 health (health 2.4 + armour 50 at the detour — the armour pool
   carries the whole bunker phase to the end). Tight but green, reproducibly (3 consecutive
   green runs through the edits).

**Result [L three runs, final tree]: the civilian survives the whole route** (`drone_round_hits`
stays EMPTY the entire walk — no guard round hits any drone; ch 0x60 never rises; the guards
end the crossing investigating far west at states 104/71 with their lines 1.7-3.1 u clear of
her), and the walk completes: ch 0x81, the computer 0x48, both geisha rescues, the Load Point,
the 07000003 hand-off, both objectives complete.

## The pin

At the stair bottom (the leg the old fail landed on) the walk now asserts, at 00 Agent: ch 0x60
OFF (the civilian alive [M 0x1827d8:98-107 — the fail reads her death channel whatever the
killer; D census #464]) and `drone_round_hits` empty (the west-end rifles never opened fire
across the lane [L the silenced-duel runs]). All temporary diagnostics from the base branch
(the stair-bottom print block, the two exposure probes, the status state extension) are
removed.

## Tests run (release, --test-threads 1)

| binary | result |
|---|---|
| walkreal_07000002_live (whole, d3 un-ignored) | **5 passed / 0 failed / 0 ignored** |
| interactables_07000002_live | 18 / 0 |
| walk_07000002_live | 2 / 0 |
| hostage_killer_07000002_live | 5 / 0 |
| hostage_save_07000002_live | 3 / 0 |
| findcel_tie_break_07000002_live | 1 / 0 |
| census_07000002_drones | 0 / 0 / 1 ignored (its own print-only dump ignore, untouched) |

No `crates/*/src/` and no `tests/common/` changes — the branch's only code-bearing diff vs
master is `walkreal_07000002_live.rs` (this file's own helpers and d3-gated arms), so no
cross-binary or `walkreal_*_live` sweep is owed beyond the 02 family above.

**The full 4-crate suite did not run this window**: this sandbox kills background jobs between
tool calls (verified twice last session in this same sandbox — the launched `python3
tools/test/suite.py` is gone on the next tool call, `suite.out` empty), and the 10-minute
foreground cap per tool call cannot cover the suite's build + run. The 02 family above is the
window's coverage.

## What's left

Nothing for this task's ignore. Carry-overs for the main agent:

1. The d1/d2 safety remains timing luck (the traced chain above); any future change that
   re-times those walks can re-expose the civilian. The new stair-bottom pin catches it.
2. The base report's live check 2 still stands (do the game's own guards 452/453 open fire
   across the courtyard at a real player? — decides whether the port's sight scope on this lane
   portal needs its own look; this round's fix works WITHIN the port's current sight model).

## Live checks for the main agent

1. On 00 Agent (fresh boot, 0x07000002): confirm the game's own guards 452/453 never fire
   across the upper-east courtyard at a player who runs the loud duel — i.e. is the port's
   hearing/Investigate convergence faithful, or does the real game keep them idler still?
   (gdbwatch on `DroneVision_AlertSound` 0x177800's share read at their drone addresses.)
2. The P99's 0x403 outer-40 silence at 41-48 u: one render/`--bond` spot at the plate
   (−35.2, 6.5, 8.2) mid-duel — confirm no alarm state change on the far-west guards in the
   real game (the port's share math rides `SFXOutputData` [D]; a fresh-boot watch settles it).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.
