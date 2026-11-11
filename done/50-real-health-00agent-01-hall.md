## Your task: `real-health-00agent-01-hall`, Double Cross part 1 can be walked with the contact on 00 Agent with real health

Branch from origin/master. walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent is ignored: Bond reaches the AND(0x11,0x12,0x13) hall gate at 85.6 hp but the door phase is ammo-exhausted (the pre-hall stock already consumed the off-path boxes). Read the full ignore message. Plan the ammo: restock on the route the level offers [D the pickup placements] before the hall, and take the three named guards with head shots from the aperture. Real input only, no top-ups, never lower damage or health; helpers gated to 07000001 and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000001_live and interactables_07000001_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-01-hall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-11T00:47Z

---
## Report (done, 2026-11-11T01:44Z)


# real-health-00agent-01-hall

Branch `s/real-health-00agent-01-hall` (cut from origin/master 784ba2d99).

## What's done

The 00 Agent double-cross walk is NOT green yet — per the task's own instruction ("Remove the
ignore when green, otherwise push the ledger"), the ignore stands with an updated ledger. The
round rebuilt the door phase around five new levers, all d3-gated real input in
`walkreal_07000001_live.rs` (no shared-file changes — `walk_to_the_exit_at` is level-local):

1. **The hoard rule** (the drain-4 ledger's next lever): the d3 pre-hall stock now runs only
   when the ammo Bond OWNS is short — the wheel's firearm clips + the carried pools (`Inventory`
   [M pickup.rs:409]) — not the held-gun measure that missed the pool the boxes fed. On this
   stream the unconditional 3-pass stock's 20-u corridor round trip took 51.0 of 100.0 (guards
   13/0/8 firing from 14-21 u) to put 16 rounds on a spare Bond wasn't holding [L the 6709 run
   trace]; armed runs now keep the legs home and the route's boxes stay offered [D the pickup
   placements] when actually short (threshold 60 rounds).
2. **The pre-clear diet** (the drain-3 lever): the named guards stay for the doorway at d3 —
   "the same kills, later, for a third of the rounds" [M NDrone2_HitDamage 0x145558 head ×100].
3. **The keep-armed lever**: `d3_switch_to_deepest_gun` — the deepest owned clip+pool through
   the real weapon-select input [M Player_Weapon 0x1a24c8] — at the loop arm and both dry
   branches. The first-armed re-arm kept a 7-round pistol while the wheel SMG sat at 32 [L the
   10501 run]; the door phase held the FISTS at wheel-empty [L the drain-4 detour dump].
4. **The reload lever**: the inline lure/fight loops never passed the reload arg — an empty
   clip clicked dry through whole windows (clip 0 spare 19 across a 900-tick lure, zero shots
   [L the 4561 run]). They now reload with the trigger up [M Player_WeaponFiring 0x1a5460's
   reload branch (gun.rs:529-535: the key reads only at the trigger level 0, view idle)].
5. **The crouch + the hold-line fight**: the hall's solid hanging lamps sit on the standing eye
   line — every standing part ray was lamp-blocked (exposed None ×6 from the threshold [L the
   1079 run]) while the guards' lower fire bones kept landing. The crouch drops the eye under
   the lamps (exposed_aim already reads the live crouch_drop) and the parts expose; the fight
   runs from the west-shifted hold line (x 7.2 → 6.5 — the deep captain's fire lane reached the
   old line [L the 13243 run]) with taps every converged tick an exposed part shows, no wound
   backing, and the threshold stand retired (both streams died in it: 37.0 [L the 7771 run],
   49.0 [L the 9191 run]). The shoot machinery's flank is refused for across-and-alive guards
   at d3 (the historical walk-into-the-hall death, [L the 8604/19098 runs]).

**The remaining wall** (the ledger's REMAINING WALL section): the deep shotgun captain #7
(0x5000038, 24 hp). His fire lane reaches every south-of-door position — his 5-7-round bursts
landed at the hold line from 10.7-11.2 u [L the 13243/13546 runs] — and the crouched exposure
windows gave body hits only (his hp 24→16 across the loop). Bond reaches the door phase at ~49
hp (the route tax: #13 25.2 in 14 hits + #0 24.0 in 2 — the d3 accuracy 1.0 + pain 1.2 the
d1/d2 walks don't pay [M tuning.rs 01-04 group]) and the trade runs out. The ledger's next
lever: pre-clear the west-corridor guards BEFORE the meet walk (the fights Bond chooses, not
the return-fire answers) so ~30 more hp reaches the door phase.

## How it was verified

- `walkreal_07000001_live` (whole binary): **7 passed, 1 ignored** (the d3 ledger) — including
  the d1 real-damage walk and the d2/Agent real-damage walk, re-verified after gating two
  stream-visible edits (the lure's exposed-part aim and the reload handling) to d3: un-gated
  they shifted the shared RNG and killed the d1 plate walk (the failure reproduced on my tree,
  absent on the stashed base [L the stash A/B run]).
- `interactables_07000001_live`: **10 passed, 0 failed**.
- The d3 test itself: 15 instrumented runs ([L] traces cited above; /tmp/d3_run*.txt).
- **The fast suite did NOT complete**: `tools/test/suite.py` was started on the branch base
  minutes after the branch push but was killed by the sandbox's memory pressure (suite.out 0
  bytes; the same OOM SIGKILL pattern the mp_stress/mp_online binaries show in this sandbox).
  The walkreal + interactables batteries above are the explicit coverage. **Re-run the suite on
  this branch before merging.**

## What's left

- The suite re-run on the final tree.
- The ledger's next lever (the west-corridor pre-clear before the meet walk) — the next round
  continues from the branch.
- Live checks for the main agent (PCSX2, fresh boot):
  1. A crouched player at the 07000001 hall door line: can he see the hall guards' parts while
     the standing eye cannot (the lamp line)? [S the surface read — the port's lamp solids
     (following_mayhew note) + the guards' lower fire bones.]
  2. Drone hearing on 07000001: does a sustained pistol burst at the hall door line draw the
     ch-0xE guards across the aperture in the real game? (`DroneFunc_SetNoiseHeard` 0x149920 /
     `HandleSoundAlerts` 0x148a30 — the port's chain exists [M drone.rs:20781+] but the
     alerted attack states have no msg::SOUND arm in the port — the game's combat states'
     0x14 handling needs the decompile read.)
  3. The guards' weapon fire-bone height vs the hall lamps' z-span (the crouch lever's ground).

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost or
context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~02:49 UTC; this report written ~02:26 UTC), `git` hashes, test outputs, and process
exit codes. No input/output token counters, no credit balance, no dollar cost.
