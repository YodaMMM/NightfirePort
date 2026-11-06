## Your task: `real-health-00agent-02-5`, 07000002 can be walked to the end on 00 Agent with real health

`walk_the_castle_all_objectives_real_health_00agent` in crates/nf-view/tests/walkreal_07000002_live.rs is still ignored, and no session has worked on it beyond one repro. Read the 02 sections of the "Report from the worker" in the queue's blocked/50-real-health-00agent-02-03-4.md. That repro died at t6833 at (-31.35, 6.46, 8.40): 66 hits for 100, of which #7 (skin 0x500003a, weapon 0x1c) landed 88.8 in 32 hits at d 3.5, fought with a dry gun, and the log read "geisha 1 stand-off 0 shotgun: 0x1c not carried". Branch from origin/master and repeat the repro first, because the guards' door and radius work may have moved it. The walk's own 00 Agent tactic is to scoop a dropped shotgun and kill a 12-hp captor with one tap, so establish from the decompile and the placement [M]/[D] what #7 and the earlier captors actually drop and when, instead of relying on the walk's timing. Then reach the geisha-1 room with a loaded gun: scoop an earlier drop deliberately, or re-order the stand-off so a drop lands before the duel. The dry-gun fix in docs/reports/door-walk-03b.md on origin/s/door-walk-03b (resupply arms before each risky leg, along covered paths only) is the same failure on 07000003, so reuse its shape. Real input only, no top-ups, never lower damage or health, and gate the changes on difficulty 3 so the other walks stay byte-identical. Remove the ignore when green. If the walk is still red, push the new ledger and the next lever in the ignore message. Run the whole walkreal_07000002_live binary and walkreal_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-02-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-06T22:08Z

---
## Report (blocked, 2026-11-06T23:48Z)

# real-health-00agent-02-5 — report

Branch `s/real-health-00agent-02-5` (cut from `origin/master` e4fb1c512). The tree touches ONLY
`crates/nf-view/tests/walkreal_07000002_live.rs` (+ the new `crates/nf-view/tests/level02_placement_probe.rs`,
an `#[ignore]`d assertion-free [D] dump) — **no `crates/*/src/` change, no shared-helper change**
(`walker.rs`/`damage_log.rs` untouched), so every other test binary is byte-identical to master by
construction; the whole `walkreal_07000002_live` binary confirms it (below).

## Status: the 02 00 Agent walk is still `#[ignore]`d, but the death MOVED one full section —
## the geisha-1 stand-off (the 03-04 blocker) is WON; the walk now dies at the courtyard sniper

## What was done (all in the 02 test file, every change `difficulty >= 3`-gated)

1. **Repro on master first, as ordered**: the 03-04 numbers reproduce EXACTLY on master e4fb1c512 —
   death at **t6833 at (−31.35, 6.46, 8.40)**, 66 hits for 100.0, #7 (skin 0x500003a, weapon 0x1c)
   the killer at d 3.5, the log reading "geisha 1 stand-off 0 shotgun: 0x1c not carried". The
   guards' door and radius work did NOT move the 02 death.
2. **The [D] placement facts** (the new probe, run with `--ignored`): the geisha-1 room is drone 5
   (skin 0x5000038, weapon 0xe, 24 hp — the bound captor at the geisha's side) + drones 6/7 (0x1c
   shotguns, 12 hp). NO placed pickup helps before the plate: the P99 ammo #369 sits at
   (−74.2, 5.6, 24.0) and the second armour at (−55.2, 5.7, 4.7), both off-route; the resupply
   before the geisha-1 room can only be drops. Guards drop their HELD weapon at the death-anim arm
   with clip·(0.25+FRand(0.5)) rounds [M DroneWeap_DropWeapon 0x17a2a0; crates/nf-game/src/drone.rs
   `drop_weapon`].
3. **Fix 1 — the corridor-drop resupply before the plate** (`scoop_drops_within`, a
   radius/y-tolerance parameterisation of the file's own `scoop_drops`; two d3-gated passes after
   the south-hall-west sweep and before the plate): the leg's four return-fire kills (#1 t4428,
   #3 t4780, #2 t5060 on the repro run) drop their rifles ON the covered corridor, and the old
   walk fought the duel on a 23-round P99. The scoop arms a 0x18 with 40 rounds
   ("armed the fullest gun 0x12 (40 rounds; carried [(18, 40), (1, 1), (2, 23)])").
4. **Fix 2 — the stand-off resupply gates on the kill** (`if !killed { continue; }`): the old
   unconditional `arm_weapon(0x1c)` hunted a not-carried shotgun through up to ~730 ticks of wheel
   cycling per round while the closing captor volleyed — the run-2/3 duel trace shows Bond going
   88.8 → 6.0 health across one such gap. A player with a loaded rifle keeps fighting; he loots
   after a kill.
5. **Result — the geisha-1 section is won**: stand-off 0 ends at t6799 with 88.8 health intact;
   the k=1 kill lands, the drop is scooped and "geisha 1 stand-off 1 shotgun: armed 0x1c"; captor
   rounds run on the shotgun (captor 0 t10701, captor 1 t11629, both killed, drops scooped); 78.0
   health into the touch zone; the zone raises; the stair-top/courtyard fights take drones 10/11
   at ZERO health cost (78.0 held t10701 → t14143).
6. **The new death — the courtyard sniper #9** (weapon 0x1e, 12 hp [D placement
   (−52.05, 6.34, −14.43)]): the stair-top aimed stand gives up on aim None (his window never
   opens from the top — the -03-2 ledger's own finding), the bottom-lip stand likewise, and the
   crossing cannot outrun his first shot: shot 1 (−48.0) lands +335..+557 ticks after the bottom
   arrival wherever Bond is (the lip, or 1-3 u into the lane), shot 2 (−30.0) is lethal at 30.0
   health. The run-7 return fire answered with the 7 in-hand rifle rounds and missed (the plain
   stand's hip cone at 15-18 u; the aim raise does not zero a non-0x400000 record [M Bullet_init
   0x12c5a0:139-170]). The runs 4-8 deaths: t14648/t14641 at the lip, t14820 3 u into the lane.
7. **The lane hooks restructured against the dwell**: the old bottom `arm_fullest` cycled a full
   unchanged wheel (~300+ ticks) standing in the kill zone (the −48 landed DURING the hooks); now
   the stair-bottom hook is the scoop only (with the y-tolerance 2.2 — the descent kills' drops
   land ON the crossing line below the lip; the run-7 drone-11 drop sat at (−40.8, 4.4, −0.9),
   1.85 u down, filtered by the old 1.5 gate), and the aimed lane-mouth duel runs AFTER the
   crossing from the spot the sniper is actually exposed from (its first live chance is this
   tree's post-leg hook; the runs so far died before it could answer).

## What's left (the next session's levers, in order)

1. **The sniper duel at the lane mouth**: the aimed stand now exists post-crossing; if his window
   opens there before shot 2, a 12-hp sniper dies to three body rounds [L the 00agent-02-03-2
   ledger]. If the mouth still shows nothing, the exposure point needs measuring (a probe pass
   along the lane printing `exposed_aim(9)` per u).
2. **Health headroom**: the stand-off k=1 volley cost 10.8; keeping 88.8 to the lane means
   48+30 leaves 10.8 alive. The k=0/k=1 gap is one volley — the 6.0 back_off ring vs the closing
   12-hp #7 is the knob.
3. **Crossing ammo**: the run-7 return fire had 7 rounds; the on-the-way scoop (run 8) took the
   lip drop at d 0.5 — verify the take lands and the fight runs on 15-27 rounds.

## How it was verified

- `walkreal_07000002_live` WHOLE (this tree, final): **3 passed / 0 failed / 1 ignored** (the
  ignored = the 00 Agent walk per its ledger) — the d1 `real_health`, d2 `real_health_agent` and
  `to_the_exit` walks are green on this tree, confirming the d3 gating keeps them byte-identical.
- The 00 Agent walk: 8 deterministic runs this session (runs 1-8, 20-95 s each, per-tree
  reproducible): master repro t6833 → the resupply runs (t7561) → the kill-gate runs (the
  geisha-1 room won, the sniper deaths t14648/t14641/t14820) → the lane-restructure runs.
- The placement probe run once (`--ignored --nocapture`): the roster + pickup dump quoted above.
- The whole `walkreal_07000003_live` binary: NOT run in the window left — no shared-helper or
  `crates/*/src/` change exists (the diff is the two 02 test files only), so its binaries are
  master's by construction; master's last full 03 state was 6 passed / 0 failed / 1 ignored
  [L the 00agent-02-03-4 report's runs on the same master-side code].
- The full four-crate suite: the background `tools/test/suite.py` start died between tool calls
  (this sandbox kills background jobs); the foreground window went to the 8 walk iterations +
  the 02 binary. Not run; the tree's only Rust surface is the one test file.

## Live checks for the main agent

1. The courtyard sniper's window: a fieldprobe/gdbwatch on drone 9's (skin 0x5000036, weapon
   0x1e) vision/exposure while Bond walks the stair bottom → courtyard lane line at 00 Agent —
   from WHERE along the lane does the game expose him (the run traces say never from the stair
   top or the lip; the -03-2 ledger says the lane mouth does)? That pins the duel stand spot.
2. The sniper's fire cadence: the trace shows shot 1 at +335..+557 after the stair-bottom
   arrival and shot 2 ~170 ticks later — confirm the acquisition trigger (the lane entry vs the
   descent noise) so the crossing can be timed against it.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
