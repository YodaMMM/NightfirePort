# 07000001: the start guard's first rounds and the contact

At the level start the contact stands near the line between start guard #0 and Bond. On the current tree the guard's rounds can kill the contact from 0.04 u off the line. Read the guard's first-attack rules (when an idle Bond at the start spot is engaged, the spread, and whether rounds stop on an ally) and compare the hit with the game's collision for the contact's body. Fix the port if the contact is hit where the game's round would pass.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-start-guard-contact`, report in docs/reports/level-01-start-guard-contact.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-08T21:19Z

---
## Report (done, 2026-11-08T21:54Z)


---
## Report from the worker (done, 2026-11-08T21:54Z)

# level-01-start-guard-contact

Branch `s/level-01-start-guard-contact` from `origin/master` (b059df948). Task: at the
07000001 start the contact (Mayhew, skin 0x4F, team 2) stands 0.04 u off the line from the
start-room guard #0 (placement #158) to Bond; on the current tree guard rounds can kill him
there. Read the guard's first-attack rules and the round-vs-body collision, and fix the port
if the contact is hit where the game's round would pass. No draw changes.

## Done

**No code fix shipped: the port does not hit the contact where the game's round would pass —
the port's contact, like the game's, is not on the line when the guard fires.** What the
session established, with the evidence:

1. **The kill does not reproduce through real input on this tree.** Three scenarios, all
   zero-input after the intro (no teleports): settled stream idle 900 ticks, the same without
   `settle_start_draw` (the fix_mission stream), and a 1.6-east sidestep then idle. In all
   three the contact takes **0 hits** (health 10.0 throughout); the guard fires 8-11 rounds;
   the closest a dead-on line passes to the contact is **0.48 u** — outside his posed boxes
   (the head box half-width 0.11, the torso ~0.3). Probe: `first_rounds_contact_probe.rs`
   (committed), the three earlier runs are quoted in the WORKLOG entry.
2. **The game's own trace shows the contact off the line before the first shot** [L
   tests/traces/01-doors.trace f+0..f+300, intro skipped at f+0]: the contact sits at his
   spawn (−7.70,−7.62) only until f+75 (state 28 = 0x1c AllyLeadInit), walks west through
   states 32/29/40, and reaches the hide spot **(−10.76,−6.73) at f+137-180** (state 31 =
   0x1f AllyLeadHide, cower anim 0x19 [M NDrone2_DSTATE_AllyLeadHide 0x15ec08 case 1 — in
   place, no teleport]). The guard is still in his start script (state 3) until ~f+150. The
   port's contact ends at **(−10.784,−6.764)** — 0.03 u from the game's hide spot — in state
   0x1f/anim 0x19. The port reproduces the game's hide walk.
3. **The guard's fire model matches the decompile** [M]: hit roll `FRand(100) < pct` → the
   aim offset at +0x200 is ZEROED (dead-on at +0x1f0); a miss takes the wobble
   (0.5·sin a, 1.5·sin(a+π/2), 1.5·sin 0.02t), rotated by RotMatrix(+0x1c0) where
   GetOpponentInfo writes **(0, yaw_to_opp, 0)** [M 0x179740:116-134; 0x138f68:70-75] — the
   port's yaw-only rotation is right. Beyond TooClose a hit also needs
   +0x270 ≥ NewSighting_TimeToHit·60 and !+0x41 [M 0x179740:103-109]; the port applies it
   (the probe's first shots: seen 61-63 → forced wobble). Cross-side landing (team 1 on
   teams 2/3) is the game's own rule [M 0x13fff0:181-232] — pinned by ally_bullets_live.
   The bullet-vs-body pick: the game's sphere r = obj+0x8C ≈ 1.05 [M 0x1eae80:91-141] then
   the posed HumanCollisionBones boxes in list order; the port adds a 2.1 broad-phase sphere
   that only admits candidates — the boxes decide, so it cannot create a false hit.
4. **The port's guard fires its first shot ~130 ticks too early — the real divergence, and
   the plausible source of the historical kills** (walkreal_07000001 "frames 289 and 407"
   + fix_mission "health −1.25"): the game's guard ends his script at f+150 into AIM_STAND
   **0x44** and holds it ~139 ticks while the aim anim (0x0a) plays, then enters the fire
   state **0x47** at f+289 and fires [L 01-doors.trace]. The port's guard reaches AIM_STAND_FIRE
   (0x68) ~10 ticks after the script end and fires at the ENTER (pin: hide at tick 177, first
   shot tick 180 — a **3-tick margin** where the game's is ~119). The port's arm
   (drone.rs:7901 `call_anim(d, 10, AIM_STAND_FIRE, 0)`) matches the game's
   `CallAnim(0,10,0,0x68,0,0)` [M 0x15ba68:129 — param_2=10 is the ANIM state, param_4=0x68
   the completion AI state stored at +0x596, routed on completion by 0x13ab58:97-107]; the
   completion routes instantly when the requested anim state equals the playing one
   (0x13ab58:97 `param_2 == uVar2` → CallFullyComplete; the port's same arm drone.rs:3064),
   so the suspect is the guard's anim state at the script end already being 10 in the port
   (or a near-zero-length 0x0a danim) — unverified.

## How it was verified

- `crates/nf-view/tests/first_rounds_contact_probe.rs` (NF_ISO opt-in, real input only):
  asserts the contact reaches the game's hide spot (−10.76,−6.73 ±0.25/0.35) **before** the
  guard's first shot, lands no round on him in a 900-tick idle engagement, and that the
  engagement is live (Bond takes damage). **Green on this tree** — the parity pin.
- Decompile reads cited inline above; the game trace tests/traces/01-doors.trace for the
  game-side order; the probe's per-shot log (muzzle, aim, seen ticks, line→contact perp) in
  the run output.

## What's left (the next session's fix)

1. **Make the port's aim-anim phase run its length**: at the 0x6000168/0x6000169 script end
   the guard must play anim state 0x0a for its danim length (~139 ticks per the trace)
   before the AIM_STAND_FIRE completion routes. Check why the port's completion routes in
   ~10 ticks: the guard's anim.state at the script end (if already 10, the
   `state == self.anim.state` instant arm at drone.rs:3064 fires), or the 0x0a danim
   lookup/length. Then check the game's 0x68 → 0x47 step (the trace's fire state is 0x47;
   the port enters 0x68) — a constant-value question per the merge lessons.
2. After the fix: re-run first_rounds_contact_probe (the margin should widen to ~100+
   ticks), every `walkreal_*_live` binary (the anim change touches every drone walk), the ai
   binaries, guard_ai_*_live, fix_guard_first_shot_live, fix_mission_live,
   walkreal_07000001/07/09/11/08, then the full suite.
3. The historical kill's exact stream (frames 289/407) is pre-2998b6a76; not reproducible on
   this tree — no walk change needed; the walks' sidestep stays as the player-honest step.

## Live checks for the main agent

1. On a fresh-boot PCSX2 run of 07000001 with the intro skipped: watch the start guard's
   (drone 0) `Drone+0x56a` (anim state) and the AI state from the script end to the first
   shot. Expected [L 01-doors.trace f+150→f+289]: AI 0x44 with anim 0x0a playing for ~139
   ticks, then the fire state 0x47. Confirm the anim state during the dwell is 0x0a (and not
   something else) — that settles whether the port's instant completion is the anim-state
   equality arm.
2. Record `Drone+0x270` (seen ticks) of drone 0 at its first shot: expected < 120 with the
   intro skipped (the sight age starts at the skip) → the game's first rounds are wobble
   rounds, matching the port's seen 61-63 first shot.

## Usage

Figures visible to me this session: the research subagent's own usage footer read from its
result: `Tokens -> prompt:252968 completion:7470 total:260438` (the Explore agent's run,
2026-11-08). My own session's token/context/cost figures: not visible to me.
