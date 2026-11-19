## Your task: `retune-nb-13-4`, 13 warm chain to the front entrance real-damage walk green again (round 4)

Branch from current origin/master; if origin/s/retune-nb-13-3 is not on master, merge it in first. Read docs/reports/retune-nb-13-3.md there and the ignore message on walk_the_warm_chain_to_the_front_entrance_and_out_real_damage. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-13-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-19T04:26Z

---
## Report (done, 2026-11-19T05:50Z)


---
## Report (done, 2026-11-19T05:50Z)

# retune-nb-13-4 — the 13 warm-chain real-damage walk is GREEN (round 4)

Branch: `s/retune-nb-13-4`, from current `origin/master` with `origin/s/retune-nb-13-3` merged in
first (it was not on master — clean merge, 16 commits). Task: the retune-nb-13-3 report's first
What's-left item — **the drain's failed-stand skip-set** — for
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (`walkreal_07000013_live`, 00
Agent d3, real damage). Real input only, no top-ups (health), never lowered damage or health,
never changed guard code; every helper change gated to the level and the difficulty; each lever
judged on its own real run.

## Result: GREEN — the ignore is removed

The walk completed on this tree: **MissionState 6**, the entrance touch ch 0x50 written, the
objective states asserted, `test result: ok` [L this session's run]. The `#[ignore]` on
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` is removed; the GREEN record in
the test documents the lever chain with the run evidence. The whole
`walkreal_07000013_live` binary runs **4 passed / 0 failed / 0 ignored** (`--test-threads=1`:
the plain walk, the d3 real-damage walk, and the Agent/Operative sisters), and
`interactables_07000013_live` runs **10 passed / 0 failed**.

## What was done — five levers, each judged on its own real run (all [L] = this session's runs)

The ledger's lever was tried first and measured; the run's own measurements then drove the chain.
Every change is file-local to `crates/nf-view/tests/walkreal_07000013_live.rs` inside
`real_damage && difficulty == 3` gates or defaulted-off params — no `crates/*/src/` files
changed, no `crates/nf-view/tests/common/` files changed.

1. **The ledger's lever — the drain's failed-stand skip-set — measured WORSE (death t5872 →
   t5253), kept as a param, and the measurement recorded.** `drain_the_wave` grew a `skip`
   param: a failed stand (+retry) pushes the drone into a `failed: Vec<usize>` and the loop
   CONTINUES to its other rounds instead of the stop_on_fail return. The real run: after the
   watch drain's failed #6 stand the drain's other rounds found **no other exposed target** (the
   [013] log shows only the #4 and #6 stands ran) and the quiet waits kept Bond statue-still in
   #6's line — **#6 advanced to d 14.6 (z 25.8) during the waits and burst 18.0 in 11 ticks**
   (t5242/5248/5253) [L]. The skip-set stays (param-gated, the east-lane drain passes false per
   the byte-identity law) because it is the right law for the door drain's other rounds, but the
   watch drain must not run it.
2. **The watch drain capped at ONE round (`max_rounds` 8 → 1).** The pick takes the lowest live
   exposed drone — #4, the balcony sniper, killed true (proven three times in the 13-3 levers
   and again here) — and the drain ENDS: no quiet-wait statue, no #6 trade. The #6 stands buy
   nothing: three caught pauses dealt 21 for a −21 fire trade (hp 24 → 3) and the 4th pause never
   came in either retry window (levers 9/10, byte-identical t5872); hp-3 #6 fired the same line
   at the crossing. The walk-on legs start at once and #6's sight-based fire meets a moving
   target.
3. **`loose_floor` at the door drain — the charger law inside the drain.** Floor-level targets
   (|dy| ≤ 2.0) run `shoot_loose` (±0.07/±0.10, the file's proven charger answer that killed #9
   at zero health cost) instead of the still-gate `aimed_stand`. The measurement that demanded
   it: at the door mouth the wave-rifle #6 (a 0x5000006 charger, floor level, d 3.6, hp 24)
   stood killed-FALSE through the 700 cap and the 1400 retry while the trade ran 46.0 → 23.0,
   and the E-leg return-fire fight then ate another −21.0 (23.0 → 2.0) for killed false — Bond
   ran the whole room phase at 2.0 hp on the fights' protection until the endgame resupply nav
   met the respawned #7's point-blank shell [L]. Non-floor targets keep the still-gate stand
   (#4's kill is the still-gate's own proven act); the room drain passes false (its #8 kill is
   the still-gate's too).
4. **The d3 exit answers the doorway mouth from leaf-open (k > 60), with a skip-set and the
   loop cap 1500.** The crossing loop's mouth-guard answer was gated at k > 300 (the stall
   law). The cap-1 loose run measured the kill zone: the respawned #7 (skin 0x5000008, weapon
   0x1c — a SPAS12, hp 12) stood IN the mouth (d 1.0) when the open window began and unloaded
   −39.0 in 21 hits into Bond (health 30.0 at the leg end) before the stall timer ever fired
   [L]. A player with a guard in the doorway shoots him the moment the leaf opens. The pick
   radius (1.6 of the mouth point) only matches guards IN the mouth, so the earlier trigger
   cannot fire at distant guards; the skip-set keeps one unfinishable mouth guard from burning
   the loop budget on repeats; the cap grew 900 → 1500 because the answers' 300-tick stands
   need the room. The d2/d1 sisters keep the stall law and the plain pick (byte-identical —
   verified by run: both sisters green).
5. **The exit ammo: the 3-pass exit fill + the deepest-gun arm.** The exit fights ran DRY —
   the mouth answers fired clip 4 spare 3 and then clip 4 spare 0 and both mouth guards
   survived; then #11 (the 0x19 pistol at (−26.4,−368.4,40.9), −3.5 a volley) took the
   crossing's 29.5 [L]. One fill pass fetched the nearest guard weapon drop (d 2.9) whose gun
   went to the INVENTORY while the held stayed 7 — the call's own ammo-rise check broke there
   [L]. The fix is the re-anchor fill's own law: up to THREE errands (the room's own box at
   (−27.2,−369.4,30.1) d ~2.5 and the fallen guards' weapon drops, each a short there-and-back
   errand via `resupply_home`), stopping when the POOL (held + the owned inventory rounds)
   reaches 25, then the watch arm's own deepest-gun cycle + reload. The green run's answers
   fired armed (clip 6 spare 10 — #11 killed at d 0.9), the lobby rifle answer ran, and the
   entrance band completed. The margin-6 dead-fetch hazards are gone on this timeline (the
   room's bottom guard — health 24, at the old #9 post — was killed by the last-guard loop;
   #4/#6/#8 dead; the door CLOSED for the errand — the mouth guards' pellets through the leaf
   measured 0.00). d3 only.

## Verification

- Byte-identical repro before any change: not needed this round — the session's first run WAS
  the ledger's lever (the skip-set), judged on its own real run; each subsequent lever landed
  on top of a committed, pushed checkpoint.
- Six real runs this session: the skip-set (t5253), the watch cap 1 (t11024 inside the room),
  the loose-floor door drain (t11203), the early mouth answer (t11694, the dry answers), the
  one-pass fill (t12180, #11's volleys), and the 3-pass fill + arm (**GREEN**, MissionState 6).
  Each run's log in `/tmp/lever1_skip.log` .. `/tmp/lever6_fill3.log` (sandbox scratch,
  uncommitted).
- `walkreal_07000013_live` (whole, `--test-threads=1`, final tree): **4 passed / 0 failed /
  0 ignored** — the plain walk, the d3 real-damage walk (un-ignored), and the Agent/Operative
  sisters. The sisters are byte-identical by construction AND by run: every lever lives inside
  `real_damage && difficulty == 3` gates or a defaulted-off param (the east-lane and room
  drains pass `skip=false, loose_floor=false`; the crossing's k>300 law and plain pick are the
  sisters'; the exit fill/arm block is d3-gated).
- `interactables_07000013_live` (whole, final tree): **10 passed / 0 failed**.
- Lesson 10 sweep: nothing under `crates/*/src/` changed (the whole change set is the one test
  file plus docs), so the grep-and-run-each-caller requirement reduces to this file's own
  binaries — both ran whole, above. `crates/nf-view/tests/common/` is untouched, so lesson 7's
  every-walkreal-binary requirement does not trigger.
- The fast full runner (`tools/test/suite.py`) on the final tree (master is an ancestor —
  verified with `git merge-base --is-ancestor`): a 560-second foreground chunk (this sandbox
  kills background jobs between tool calls — the task message's `nohup … &` shape cannot
  survive here) consumed ~1.7 GB of the remaining disk, produced **zero output lines** (the
  build phase had not reached its first test summary), exit 124. With 1.8 GB left and the
  13-2/13-3 measurements (~2 GB past what remains for the 1112-binary nf-view set), continuing
  risks ENOSPC — the same measured outcome as the two previous rounds, recorded instead of
  repeated. The binaries the task names ran whole instead (above).

## Live checks for the main agent

The 13-3 report's two questions are answered by this session's runs; nothing further is
required for this task:

- (a) #6's box opens from the watch position AND from the door mouth (both drains stood on
  him), but the still-gate's 4th pause never comes: levers 9/10 (13-3) plus this session's
  runs. He is a CHARGER — when Bond stands still he advances (d 14.6 in the skip-set run) and
  bursts (18.0 in 11 ticks). The green path never stands on him.
- (b) #6's line DOES reach inside the security room: the fights killed him INSIDE the room at
  the BodyArmour shelf (t8924, killed true, zero cost) in the cap-1 run. The green path is not
  exposed to it — the room fights answer him at zero measured cost.

## What's left

Nothing for this task. The walk is green and un-ignored; the sisters and the plain walk run
green on the same tree. The next session inherits a green 13 chain; if the driver wants the
suite's total line, it needs a sandbox with ~3 GB more free disk than this one (the
13-2/13-3/13-4 measurements agree).

## Usage

Not visible to me — this session's environment shows no token, credit or context-window
figures to copy.
