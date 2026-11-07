## Your task: `real-health-00agent-03-6`, 07000003 can be walked to the end on 00 Agent with real health

Branch from origin/s/real-health-00agent-03-5, merge current origin/master, and read that branch's report in the queue's blocked/50-real-health-00agent-03-5.md plus the ignore message on `walk_the_full_route_and_complete_every_objective_real_damage_00agent`. The walk now reaches t404475. Its main losses are #17's point-blank burst (60.0 in 4 volleys, ticks 14353-14576) and #9 (28.8, ticks 7734-7845), both inside the fights' 240-tick post-fight cooldown. Standing pre-emption is already proven worse, so do not repeat the runs that report lists as reverted. Build the lever it names: an opt-in walker.rs fights flag that re-engages at once when the cooldown would delay refighting a guard within 8 u who is hitting Bond. It must default off, so every other walk stays byte-identical. Then read the new ledger and take the corridor cluster, the (35.6, 8.7) door stall and the exit-lawn #16 volley in the order the ledger gives. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live, and because walker.rs changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T02:53Z

---
## Report (done, 2026-11-07T03:59Z)

# real-health-00agent-03-6 — report

Branch `s/real-health-00agent-03-6` cut from `origin/s/real-health-00agent-03-5`, then
`origin/master` merged clean (merge commit e75efe5c2; the merge touched only nf-view test
files and docs). **Status: the 00 Agent walk is still RED and keeps its `#[ignore]`** — but
the driver's named lever is built, proven on the ledger, and the death moved from t404475 to
t410740. The new ledger and the next lever are pushed in the ignore message.

## 1. The levers built (both OPT-IN, default off, lesson-7 flags)

The fights machinery lives in `crates/nf-view/tests/common/damage_log.rs` (the `Walker`
impl's `return_fire`/`stand_and_fire`); the flags sit beside the existing opt-ins
(`escalating_cooldowns`, `finish_fights`, `fire_through_disabled`, …). Both are set ONLY by
the 07000003 00 Agent test (`difficulty >= 3` block) — every other walk is byte-identical by
construction (the added checks are flag-gated; the flags default `false` via
`..Default::default()`).

1. **`close_hitter_reengage`** — the driver's lever, extended with the pick half the ledger
   demanded:
   - the cooldown lift: the flat 240-tick failed-fight cooldown does not delay re-fighting a
     guard within 8 u who is CURRENTLY hitting Bond (a player_hit on him in the last 240
     frames — the shooters window). The 03-5 ledger's #17 case: the 14171 fight answered
     killed-false, his 4-volley 60.0 burst ran ticks 14353-14576 inside the wait, the
     re-fight killed only at 14690 [L the NF_DMG_TRACE runs].
   - the close-hitter pick fallback: such a guard (within 8 u, landing rounds) is picked EVEN
     when no box's centre ray clears the world — his pellets reach Bond through the same gap
     the eye ray cannot reverse. Without this the fights never answered the servants'-ground
     shotgunner #5 at all (run 3/4 below).
2. **`dry_fight_resupply`** — when a fight is wanted and the IN-HAND gun is dry (clip+spare
   0), run ONE resupply pass before the silent give-up. The in-hand-dry test matters: the
   loose `armed` inventory scan passed on rounds the weapon wheel could not reach in time,
   so the fight opened on `switch_to_gun`'s 12-cycle stand and #5's volleys finished Bond
   mid-draw (run 4). Only the `!armed`/in-hand-dry arm is gated — NOT the unseen-fights
   mode's pre-fight low-ammo and post-fight restocks (see run 2).

## 2. The runs (all `NF_DMG_TRACE`, on the merged tree)

| run | change | result |
| --- | --- | --- |
| 1 | the cooldown lift only | death **t410501** (23.09,6.37,10.27) — 39 hits for 100: #17 **36.0** (was 60.0; the re-fight killed at 14615 vs 14690 — the lever works), #9 28.8 unchanged, then the servants' ground: the 410384 fight emptied the 0x12 ("cycling weapons (0) (weapon 0x12 clip 0 spare 0)"), #5's point-blank shotgun volley (20.8 at 410390, d 3.8) landed unanswered (no pick — his boxes never clear the eye ray), 111 ticks to death |
| 2 | + `with_real_damage_unseen_fights` (the built mode for both holes) | **WORSE, reverted** — the mode's extra restock arms re-directed the flank nav back to the moat drops (3000-tick caps x5, STUCK panic) and the fights woke the dodger #8 (killed false x5, the 03-5 run-7 failure mode) |
| 3 | + `dry_fight_resupply` (the `!armed` arm only) | same t410501 death — the arm never ran: the pick gate (`visible`) returns BEFORE the armed check, #5 is invisible from the wedge spot |
| 4 | + the close-hitter pick fallback | same t410501 death — the pick succeeded but `armed`'s inventory scan found unreachable rounds, so no resupply; the fight opened on the dry wheel ("cycling weapons (0)") and died mid-draw |
| 5 | + the in-hand-dry resupply gate (the final tree) | death **t410740** (24.04,6.37,12.89) — **21 hits for 100**: #17 36.0 + **#3 35.2** + #9 28.8 = exactly 100. #5 killed true 410290, #4 killed true 410181 (both former bleeders now 0) |

## 3. The new ledger (run 5) and the next lever

| threat | damage | mechanism |
| --- | --- | --- |
| drone #3 (skin 0x5000038, weapon 0xe, hp 18) | **35.2 in 4 hits** (4.0 + 12.0 + 12.0 + 7.2) | the servants'-ground captain at d 5.3 behind the same corner: two return-fire fights answered him (410433 killed false — dry, 7 ticks; 410719 killed false — 280 ticks during the dry-resupply nav, the taps waiting on an aim the eye ray never found) and the stands ate his 12.0 volleys while waiting. The unseen stand's aim-None give-up (60 ticks, non-lane) is shorter than the corner-round; the creep (`unseen_fight` + `back_off 4.0`) needs ~150+ |
| drone #9 (the mid-courtyard wedge) | 28.8 in 8 hits | unchanged: his volleys (7734-7845) run at d 10.3 — OUTSIDE the 8-u close-hitter gate; the pellets come through the closed lane doors the eye ray cannot reverse |
| drone #17 (the moat west end) | 36.0 in 2 hits | the residual before the re-engage arms (two 18.0 hits land before the first gate tick past his first volley) |

**Next lever** (in the ignore message): give the close-hitter unseen stand a longer aim-None
tolerance (the #3 fights need ~150+ to round the corner and land on hp 18), and consider
widening the close-hitter radius past 8 u to cover #9's 10.3 door-gap wedge. The corridor
cluster, the (35.6, 8.7) door stall and the exit-lawn #16 volley are still unexercised at d3
— the walk has never passed the servants' ground on real health. Standing pre-emption is
STILL proven worse (03-5 runs 2-8; this round's run 2 adds the unseen-fights restock wedge).

## 4. What ran on the final tree

The four required binaries (all green, the same counts as the 03-5 baseline — the d1/d2
sisters are byte-identical, the flags are d3-only):

| binary | result |
| --- | --- |
| `walkreal_07000003_live` whole | **6 passed / 0 failed / 1 ignored** (270.6 s) |
| `walkreal_07000002_live` | 3 passed / 0 failed / 1 ignored (62.4 s) |
| `walkreal_07000004_live` | 10 passed / 0 failed / 0 ignored (9.6 s) |
| `interactables_07000003_live` | 9 passed / 0 failed / 0 ignored (75.9 s) |

The shared-code sweep (every `walkreal_*_live`-class binary that includes the fights code,
run in foreground batches — the sandbox kills background jobs between tool calls):

- Green: `walkreal_07000001_live` 8/0 (41.7 s), `walkreal_07000005_live` 14/0 (52.8 s),
  `walkreal_07000006_live` 6/0/1 (16.4 s), `walkreal_07000007_live` 6/0 (13.8 s),
  `walkreal_07000008_live` 6/0 (100.0 s), `walkreal_07000009_live` 8/0 (10.9 s),
  `walkreal_0700000a_live` 13/0/2 (35.6 s), `walkreal_0700000b_live` 7/0 (28.7 s),
  `walkreal_0700000c_live` 3/0/1 (23.1 s), `walkreal_0700000d_live` 2/0/1 (26.1 s),
  `walkreal_07000013_live` 3/0 (29.1 s).
- **SIGKILLed by the sandbox OOM** (dmesg oom-kill; ~3.4 GB RSS on a 4 GB box — environment,
  not the diff: the flags are off in those walks, the flag-off paths are unchanged):
  `walkreal_07000011_live` (x2), `walkreal_07000014_live`, `walkreal_07000015_live`.
- Not reached in the window: 07000016, 0700001b, 07000041, 0700004a, 11_route, the 0c
  barrelrow/eastroute variants and the probe binaries; the four-crate full suite did not fit
  (the background start is killed between tool calls; the sweep above is the foreground
  batch substitute).

## 5. Live checks for the main agent

None new — the ledger came from the sim's own damage log (`NF_DMG_TRACE`), which is complete
for these questions. The 02-03-4 round's door-9 field check (does the game's own Bond open
doors 6/9 on the east corridor, and do #2/#19/#18 hold it) still stands if the main agent
has a window.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (nothing to copy).
