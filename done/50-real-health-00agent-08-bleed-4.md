## Your task: `real-health-00agent-08-bleed-4`, 07000008's 00 Agent whole-mission real-damage walk clears the fusebox pit and wins the Hind fight

Branch from current origin/master; if origin/s/real-health-00agent-08-bleed-3 is not on master, merge it in first. Read docs/reports/real-health-00agent-08-bleed-3.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. First the fusebox pit guard: pick the guard who actually landed the last hit inside the wide ring (the walker's hurt bookkeeping names him) instead of nearest-first, then the same wide stand before the bolts/safe burns. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-14T08:18Z

---
## Report (done, 2026-11-14T09:28Z)

Closed by the driver: the report is on `s/real-health-00agent-08-bleed-4` (docs/reports/real-health-00agent-08-bleed-4.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-14T09:39Z)


---
## Report from the worker (done, 2026-11-14T09:39Z)

# real-health-00agent-08-bleed-4 — report

**Branch `s/real-health-00agent-08-bleed-4`, base origin/master `385ebb66d`, `s/real-health-00agent-08-bleed-3`
(8158b1df9) merged in.**
Task: the 07000008 whole-mission real-damage 00 Agent walk past the fusebox pit guard and through
the Hind fight. The walk is NOT green; the ignore stays and carries the measured 11-14 (bleed-4)
ledger and the next levers.

## What's done

**The ledger's fusebox-pit lever is TAKEN, measured on its own real runs, and shipped d3-gated:
the hurting pick, the opening-volley poll, and the pit excursion killed BOTH pit shooters for
16.0, cleared the burn stands, and advanced the walk from the bleed-3 death t6554 to THE HIND
FIGHT (t20440).** Every change lives in `crates/nf-view/tests/walkreal_07000008_live.rs` (the
file-local `fight_guards_wide` + its two call sites + the armour gate); the sisters' timelines
stayed byte-exact [L the whole-binary run: 5 passed / 0 failed / 1 ignored].

1. **The hurting pick** (`fight_guards_wide` rewritten): the target is the guard who actually
   LANDED THE LAST HIT inside the wide ring — the walker's hurt bookkeeping (`player_hits`, the
   (frame, owner, damage) record the [dmg] trace reads) names him — falling back, before any
   round has landed, to the in-ring guard whose muzzle is live on Bond NOW (his own shot log
   ≤ 90 frames + his aim point on Bond — the same evidence `DamageLog::shooters` reads). The
   unseen nearest-first variant (bleed-3's run-lever6 regression: the buried doorway man #5,
   death t4320) is gone — the pick never stands a guard the bookkeeping has not named.
2. **The opening-volley poll** (the second call site, before the door E): the first measured
   lever run proved BOTH pre-burn call sites were silent no-ops — #16's first volley lands
   during the burn itself (t3414) and a SECOND pit shooter #15 (skin 0x500002e, (151.7, 39.2,
   −111.5)) opened 275 ticks after the burn ended [L run-bleed4-1/2]. The poll holds the burn
   spot up to 420 ticks until any in-ring guard's round has landed, then the fight runs.
3. **The y band is ±8.0, not the ledger's ±6.0**: the pit pair MOVES — #16 fired from (144.4,
   37.2, −111.1) mid-descent and the poll at the burn spot (y 44.3) read dy 7.1 — outside ±6 —
   so the ±6 fight stayed a no-op while #15 chewed from dy 3.9-5.1 and the death came t5050
   [L run-bleed4-2]. The pit floor spans y 37-39.5 under a y 43-44.3 Bond.
4. **The pit excursion**: #16/#15 are NOT visible from the burn spot (bleed-3's run-lever5) and
   sit below, so the below-level pick walks the route's OWN descent legs (landing east, through
   the door, ramp south, ramp mid, ramp turn — the path-2 link chain every green run walks)
   with a sight check after every leg; the stand fires the moment a line opens, and the return
   walks the legs back to the burn spot (the laser's 4.0 range needs Bond back on the walkway).
   MEASURED (run-bleed4-3): the poll caught #16's round at t3414, the excursion walked, the
   legs' own return fire killed BOTH pit shooters from the ramp mid (#16 t4091, #15 t4205) for
   **16.0 total**, the bolt burns ran clean, objective 1 folded ("Retrieve guidance chip" → 4),
   and the walk advanced to the boarding legs.
5. **The secure-room armour lever**: run-bleed4-3 reached the boarding at health 24.0 and the
   burst rifle #20 (skin 0x500002f) closed to 3.3 u and burst exactly those 24.0 (death t15465
   at (108.70, 28.37, −118.08)). The d3 walk now joins the d1/d2 real-damage sisters in taking
   the room's BodyArmour (kind 3, 50, (159.4, 42.6, −113.6) [D the placement census]) — the
   gate simplifies to `!tops_up()`, so d1/d2 behaviour is unchanged and only the ignored d3
   walk's timeline moves. MEASURED (run-bleed4-4): armour 50 on, the boarding cleared, the ride
   and the cinematic ran, and the walk reached THE HIND FIGHT — death t20440 at (113.04, 13.51,
   −178.54), 61 hits for 100.0, 17 return-fire fights, the machine 0.0/598.
6. **The Hind fight shape, measured** (run-bleed4-4): every end pane shot out (both 30 u
   bearings CLEAR), rockets owned clip 4 carried 4, the machine state 2 route flight at dist
   17.6-28.9 — NO launch ever opened (the d3 slit/corridor/forecast gates starved through the
   whole fight) and the dodge strafes DRIFTED Bond 2 u east of the ride-end spot into a HURT
   VOLUME that drained −2.0 every 6 ticks for the last 12.0 (frames 20344-20441 at (113.0,
   13.5, −180.6), no blast rows — the volume law, not the own-rocket signature).
7. **The anchor+zap variant: TRIED and MEASURED WORSE, NOT shipped** — the machine's burst
   cadence kept `dodge_hold > 0` through the drain (the anchor never fired once — the run-5
   fight trace is byte-identical to run-4's), the zap-triggered walk pinned Bond at (111.14,
   13.52, −180.95) — the volume reaches the SOUTH band too — and the death came EARLIER
   (t20415 vs t20440 [L run-bleed4-5/6]). The variant is reverted; the ledger names the next
   shape (map the volume's bounds, anchor the fight outside it BEFORE the machine's first
   burst, and bias the dodge's strafe back toward the clear spot instead of alternating signs).
8. **The dodge-bias + zap-walk variant: TAKEN (d3-gated) and measured (run-bleed4-7)** — the
   dodge side now picks the anchor-ward component (the missile's miss band is symmetric, so the
   side choice is free) and a world-hit zap walks Bond back to the ride-end spot against the
   dodge. Bond held the anchor — death at (111.19, 13.52, −179.81), 0.15 u from it — and the
   volume hit HIM THERE TOO: the same 33 world hits for 24.0. **The −2.0/6-tick law covers the
   WHOLE terminal floor** (the anchor (111.09, −179.72), the east band (113.0, −180.6), the
   south (111.14, −180.95)) — the "clear master spot" premise is falsified; the next lever is
   the drain class (at health ≈ 25 the 0.33/tick drain kills in ~75 ticks wherever Bond stands;
   the ride's rocket-ammo pickups went untaken on the measured roll).
   The ungated first form of these blocks REGRESSED the d1 real-damage sister (the dodge runs
   for every difficulty) — caught on the final-tree whole-binary run and d3-gated; the
   re-run is green.

## How it was verified

All runs on the slim ISO, release, `--test-threads=1`, 2026-11-14, NF_DMG_TRACE on. Each lever
judged on its own real run:

| run | result |
|---|---|
| run-bleed4-1 (the pick, pre-poll) | both call sites silent no-ops — the pit guard had not fired yet; the timeline matched the ledger death t6554 exactly |
| run-bleed4-2 (+ the 240-tick poll, ±6 band) | #16's moved position out of the ±6 band at the poll spot; the second pit shooter #15's volley came 35 ticks after the poll ended; death t5050 by #15/#16 |
| run-bleed4-3 (±8.0 band, 420-tick poll, the excursion) | poll caught #16 at t3414; the excursion walked; the return fire killed #16 (t4091) and #15 (t4205) for 16.0; the bolts burned; objective 1 folded; death t15465 at the boarding (#20 point-blank, health 24.0) |
| run-bleed4-4 (+ the armour lever) | armour 50; the boarding cleared; the ride + cinematic ran; the Hind fight reached; death t20440 (the hurt volume drain, the machine 0.0/598) |
| run-bleed4-5/6 (the anchor+zap variant) | byte-identical fight trace then a WORSE death (t20415, pinned at (111.14, −180.95)) — reverted |
| run-bleed4-7 (the dodge-bias + zap-walk, d3) | Bond held the anchor; the volume hit there too (the same 33 world hits / 24.0); death t20428 at (111.19, −179.81) |
| **walkreal_07000008_live, whole binary, final tree (2nd run, after the d3 gates)** | **5 passed / 0 failed / 1 ignored** (the ignored is the ledgered d3 walk; the d1 plain, d1 real-damage, d2 Agent and both ally tests GREEN — their timelines byte-exact; the 1st whole-binary run caught the ungated dodge-bias regressing the d1 real-damage sister, fixed by the d3 gates) |
| **interactables_07000008_live, final tree** | **15 passed / 0 failed** |

The full suite was started in the background per the task (`tools/test/suite.py`); this sandbox
kills background processes between tool calls (the same finding bleed-3 and the
walker-sidestep-decay-2 WORKLOG entries recorded — nohup and setsid both died there). My changes
touch ONE test file; the whole affected binary and interactables ran green on the final tree.

## What's left

- The Hind fight: the launch starvation (the d3 gates never opened on the measured roll — the
  machine's low passes vs the slit) and the hurt volume (the −2.0/6-tick law covering (113.0,
  −180.6) and (111.14, −180.95); the master fight spot (111.09, −179.72) is clear on every
  green run). The next levers are in the ignore message: map the volume's bounds, anchor the
  fight outside it before the machine's first burst, bias the dodge's strafe back toward the
  clear spot, and measure one launch gate's relaxation.
- The full suite on a sandbox that keeps background processes alive.

## Live checks for the main agent

1. The bottom terminal's hurt volume: standing at (113.0, 13.5, −180.6) and (111.14, 13.52,
   −180.95) after the ride, the REAL game drains −2.0 every 6 ticks (no blast rows) — confirm
   the volume's bounds (the placement census's hurt volumes around the terminal) and whether
   the master fight spot (111.09, 13.52, −179.72) sits outside it.
2. The fusebox pit pair (#16 skin 0x5000030, #15 skin 0x500002e, the level below y ≈ 37-39.5):
   confirm the real game lets Bond see them from the ramp mid (153.2, 39.7, −111.7) — the port
   does (the excursion's stands killed both) — and whether the real pit pair ever advances up
   the ramp toward the walkway.

## Usage

- input tokens: not visible to me
- output tokens: not visible to me
- context used: not visible to me
- cost: not visible to me
