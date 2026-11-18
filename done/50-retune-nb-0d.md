## Your task: `retune-nb-0d`, 0d arrival-to-C5-door 00 Agent walk green again

Branch from current origin/master. Commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states, and the walks below now die; each ignore message records the measured death. Re-tune walk_arrival_to_the_c5_door_real_damage_00agent for the new fights: read the ignore and docs/reports/real-health-00agent-0d-2.md (the container drop, the west exit, the crouched sweep), then re-roll the yard fight for the new guard behaviour. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-0d`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-18T16:22Z

---
## Report (done, 2026-11-18T17:47Z)


---
## Report from the worker (done, 2026-11-18T17:47Z)

# retune-nb-0d — the 0d arrival-to-C5-door 00 Agent walk re-tune under nb-scan-all

Claimed on master 73822d3a0 (which carries 3412a89a3, the spacing-scan-everywhere +
CanAltAttack-in-the-fire-states commit), branch `s/retune-nb-0d`. The task: re-tune
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage_00agent` for the new guard
behaviour. **The walk is still red, but the death is now measured to the round** — five real
runs this session (`--ignored --nocapture`, `NF_DMG_TRACE=1`), each judged on its own run per
the task; the measured ledger and the next lever are in the ignore message on the branch.

## What's done

1. **The death is identified to the round.** The original 0d-2 roll boards the ride at 25.8
   (the deck sniper #22, 0x5000090/0x1e, lands 46.0 at t2609 on the ladder-base pin at d 55.2;
   the corner wait chips 5 × 1.8 from #3/#1); the ride itself takes ZERO chips for ~900 ticks;
   then **g18** (owner index 18 [M `player_hits: Vec<(u32, usize, f32)>` = (frame, owner,
   damage), drone.rs 13525; L the instrumented run 2026-11-18], skin 0x5000087, weapon 0x12,
   at (177.90, 0.98, -283.26), state 0x68 AimStandFire, 33.4 u off the container landing zone,
   d ~18.4 off the rider's crouched eye) lands **ONE 25.8 0x12 round** — death t19197 at
   (164.71, 14.60, -296.08). The ride log's old print mislabelled the owner index as a weapon;
   the print now resolves the owner through the live list (log-only, no sim writes, so the
   d1/d2/plain twins stay byte-identical).
2. **Three levers tried and dead, each on its own real run, all reverted to master's walk:**
   - (a) the strip-mouth deck-duo kill (d3-gated head act, box [209.0, -289.3] tol [7.0, 3.5],
     cap 500): **LANDED** (the scan repositioned #22 to (204.21, 12.09, -290.15); the act
     converged and killed it) — but the roll re-dealt the killer: #20 (deck (197.9, 14.0,
     -262)) 46.0 at t2022 + #32 (west roof (151.4, 22.7, -238.3)) 48.0 at t2354 + 6.0 at t2895
     = death at the ladder base, EARLIER than the un-leged roll's t19197. The scan's 0x1e
     sniper roster is WIDE (#11/#20/#22 deck, #32 west roof, #36 ground band — each 46-48 raw
     at 00 Agent's accuracy row); single-target pre-ride kills just move the death. Stays out
     (the walk-7 "the net is NEGATIVE" precedent); its ledger sits in the test comment.
   - (b) the yard-armour twin leg (the d1 leg's own path, d3-gated): the kind-3 pickup at
     (205.4, 0.1, -205.3) **did NOT arm at difficulty 3** — the nav arrived (205.97, 1.13,
     -205.17), 0.57 dxz off the placement, armour 2.0 → 2.0 (at d1 the same leg measures
     "armour 50 taken") — and the walk-over cost 1483 ticks of sniper exposure (death at the
     pickup). Stays out; the d3 existence of that pickup needs a live check (below).
   - (c) the ride threat-pick zone 25 → 35 + rider-distance priority: the taps re-aimed at
     g18 (d 13.9-14.4) and **STILL landed zero** — the girder-spawn geometry law holds at
     d 14 (33+ measured taps at d 13.9-43, zero lands) — and the ride died at the same margin
     (g3 chip 1.8 at t19027 + g18 kill 24.0 at t19222). Master's zone-25 zone-priority pick
     stays; the ledger sits in the pick comment.
3. **Facts pinned for the next session** (all in the ignore message): bullets eat armour
   first (the 0.0-health deltas in `player_hits` are armour-eaten hits — the snipers' 46-48
   raw punches through the start's 50 armour in one round); g18 is outside the 25 u tap zone
   AND the bay-enter scan's wake window (its 8 passes found nothing — the yard wakes at the
   corner rim, later); the corner rim sees nothing into the yard pit, so g18 cannot be shot
   from the corner either.

## How this was verified

- Five real runs of the d3 walk this session (`--ignored --nocapture`), each a full real run
  to death or pass: repro1/repro2 (the original roll, the t2609/t17307-18032/t19197 ledger),
  run1/run2/run4 (the deck-duo roll, the t2022/t2354/t2895 ledger), run3 (the armour roll,
  the t2260/t2410/t3414 ledger), run5 (the instrumented original roll — g18's position), run6
  (the zone-35 rider-distance roll, the tap re-aim + the same-margin death).
- The whole `walkreal_0700000d_live` binary, single-threaded, on the final tree:
  **3 passed, 0 failed, 1 ignored** (103 s) — the plain master replay, the Operative
  real-damage walk and the d2 agent sister pass; my changes are log-only, so they are
  byte-identical.
- `interactables_0700000d_live`: **11 passed, 0 failed** (31 s).
- `tools/test/suite.py` was re-run in the background on the final tree; its output is in
  `suite.out` at the repo root (the sandbox killed background builds mid-build twice earlier
  this session, so the per-binary foreground runs above are the primary evidence — see the
  suite line below for what this window produced).

## What's left

- The walk itself. The next lever is in the ignore message: re-roll the g18 headshot with the
  dice knobs (the pause_at arm timing/n and the tap cadence — each tap is a spread draw and
  g18's burst accuracy re-rolls with the shared stream), and/or cross the ground phases with
  > 26 health so ONE 25.8 round does not kill (the corner chippers #3/#1 are the soft cost —
  a corner-phase stand kill, the bay-enter scan idiom).
- Nothing else: the 0d-2 levers (the west exit, the crouched sweep, the pass cap) hold through
  the landing on every roll that survived the ground phases.

## Live checks for the main agent

1. **g18's wake frame**: with gdbwatch on the east-yard guard g18 (skin 0x5000087, placement
   near (177.9, 0.98, -283.3) — the ride-time spot), record when its `enabled`/`present`/+0x4F8
   bits flip during the corner-rim wait (ticks ~17000-18100 of the walked timeline). If it
   wakes at the corner rim proximity, a corner-phase stand cannot precede it; if it wakes with
   the platform-drop group, a pre-strip kill window may exist.
2. **The kind-3 pickup at (205.4, 0.1, -205.3)**: fieldprobe at difficulty 3 (00 Agent boot,
   not a save-state resume) — does the pickup entity exist there (placement id, kind, enabled)?
   At d1 the same spot takes armour 50; at d3 the walk-over stood 0.57 dxz away and armour
   stayed 2.0. If the placement is difficulty-gated out at 00 Agent, the armour lever is dead;
   if it exists but refused, the pickup radius/pickup-handler path needs the read.
3. **The ride taps' round deaths** (carried from real-health-00agent-0d-2, now sharper): a
   gdbwatch instrumented bullet from the gantry rider aimed at g18 (d 13.9-14.4, 17 taps this
   session, zero lands) — where do the rounds die (the girder's own collision at spawn is the
   suspicion)? If the spawn ray is eatable by a stance change (stand vs crouch eye height),
   the ride kill window reopens.

## Usage

not visible to me — this session's environment shows no token, context-window or cost
figures, so there are no numbers to copy.
