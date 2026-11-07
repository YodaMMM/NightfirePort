# real-health-00agent-03-7 — report

Branch `s/real-health-00agent-03-7` cut from `origin/s/real-health-00agent-03-6`, then
`origin/master` merged clean (the merge touched nf-view test files and docs only).
**Status: the 00 Agent walk is still RED and keeps its `#[ignore]` — same death tick
t410740 — but this round MEASURED the driver's two named levers and killed the wrong
theory in the 03-6 ledger.** The new ledger and the next lever are in the ignore message.

## 1. What was built (both OPT-IN, default off, lesson-7 flags, set only by the d3 block)

In `crates/nf-view/tests/common/damage_log.rs`:

1. **`close_hitter_stand_ticks: u32`** (0 = off) — the close-hitter unseen stand's aim-None
   give-up widens from the master 60 ticks to this (set: 180). The 03-6 ledger said the #3
   fights needed ~150+ ticks "to round the corner".
2. **`close_hitter_radius: f32`** (0 = off) — both close-hitter gates (the pick fallback and
   the re-engage lift) widen from 8.0 u to this (set: 12.0), for #9's 10.3 u door-gap wedge.
   The landing-rounds gate (a player_hit on him in the last 240 frames) still applies.

Both default to the exact master behaviour (`0.0f32.max(8.0) == 8.0`, the tolerance block's
else branch is the master 60), so every other walk is byte-identical by construction.

## 2. The runs (all on the merged tree, `NF_DMG_TRACE`/`NF_RF_TRACE`)

| run | change | result |
| --- | --- | --- |
| 7a/7b | the two levers above | **same death t410740 at (24.04,6.37,12.89), 21 hits for 100** — #17 36.0 + #3 35.2 + #9 28.8, exactly the 03-6 split. The differences the levers DID make: #9 is now answered at the mid courtyard and **killed true at 7890** (his 28.8 landed at 7734-7845, BEFORE the fight — a fight cannot undo pre-fight damage); #3 unchanged |
| 7c | + `aimed_fights` (the zero-spread AIM-button stand) | **WORSE, REVERTED** — death **t8342** at (11.58,6.46,-20.83), 40 hits for 100, 2 fights: the mid-courtyard #10 fight answered killed false while the stands held Bond in the open. The aim button rides the port's free-aim **cursor** path for these guns too — the round leaves along the view PLUS the cursor offset (the 0c-sniper scope-only reading was [S]; disproven [L the 7c run]) |

## 3. The new ledger (the 7b run's NF_RF_TRACE) — the 03-6 theory was wrong

- **#3's fights never lost the aim.** BOTH fights' t0 lines show `aim Some` — the eye ray
  was OPEN at both fight starts; the "corner the eye ray never found" never bound. The
  `close_hitter_stand_ticks = 180` widening is a measured no-op on this death (kept: it is
  the safer bound and free).
- **#3's fights died on AMMO and SPREAD.** Fight#3a (clip 5 spare 0, err 0.700): converged
  in ~9 ticks, 1 hit for 6.0 (24 -> 18), dry. `dry_fight_resupply` fired — but the pass was
  INTERRUPTED by its own nav's return-fire gate: fight#3b (label `resupply 0 nav 1`) ran on
  a clip-1-spare-1 owned gun — the taps converge (err 0.228 -> gate in 3 ticks) but the hip
  cone at 5.3 u MISSES — 2 taps, 0 hits, dry, killed false — and #3's next volley (~107
  tick cadence: 410426/410526/410633/410740) finished 7.2 health. The pass's own
  close-stretch to the drop at (24.2,5.2,14.0) never ran: Bond stopped 1.26 u short when
  the fight fired.
- #17's 36.0 is the pre-engage residual (the burst lands inside the flat wait before the
  re-engage; 14171 killed false -> 14615 killed true).

**Next lever** (in the ignore message): make `return_fire` STAND DOWN while
`self.dmg.resupplying` is true (a player mid-grab finishes the grab and THEN answers —
fight#3b would never have burned the 2-round gun mid-pass, and the pass completes its four
nearest-first attempts: the (24.2,5.2,14.0) drop, then #5's rifle drop at (24.1,14.3)),
so the leg's re-fight on #3 runs with a real magazine. The hip-cone miss at 5.3 u then
needs the stand to close distance while dry (the creep machinery exists) — or shave #17's
pre-engage residual instead (the moat answer must open one volley earlier). The corridor
cluster, the (35.6,8.7) door stall and the exit-lawn #16 volley are still unexercised at d3.
Standing pre-emption is STILL proven worse (03-5 runs 2-8).

## 4. What ran on the final tree

The four required binaries (same counts as the 03-6 baseline — the d1/d2 sisters are
byte-identical by construction, the new fields default to master behaviour):

| binary | result |
| --- | --- |
| `walkreal_07000003_live` whole | **6 passed / 0 failed / 1 ignored** (310.6 s) |
| `walkreal_07000002_live` | 3 passed / 0 failed / 1 ignored (80.4 s) |
| `walkreal_07000004_live` | 10 passed / 0 failed / 0 ignored (11.1 s) |
| `interactables_07000003_live` | 9 passed / 0 failed / 0 ignored (88.5 s) |

**The rest of the sweep and the fast runner did not fit the window.** The background
`tools/test/suite.py` start is killed between tool calls in this sandbox (the 03-6 round
hit the same wall; the nohup'd process was dead the next tool call, `suite.out` empty).
The shared-helper risk this round is bounded: `damage_log.rs` changed, but the only new
code paths read the two new fields (default 0 -> the exact master constants) and the walk
test file is d3-only. The other `walkreal_*_live` binaries were not re-run — the next
round should run them per lesson 7 (about 3 minutes each).

## 5. Live checks for the main agent

None new — the ledger came from the sim's own damage/ray traces (`NF_DMG_TRACE`,
`NF_RF_TRACE`), which are complete for these questions. The 02-03-4 round's door-9 field
check (does the game's own Bond open doors 6/9 on the east corridor, and do #2/#19/#18
hold it) still stands if the main agent has a window.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (nothing to copy).
