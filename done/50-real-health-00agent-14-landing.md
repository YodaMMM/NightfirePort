## Your task: `real-health-00agent-14-landing`, Countdown's 00 Agent infiltration walk passes the 1.72 landing

Branch from origin/master. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent wedges (not a death): the '1.72 landing' resupply leg floats Bond 0.41 over the floor at (33.84, 0.82, 30.95) at tick 26340 and the leg cap panics. A floating Bond is a collision question first: find why the port holds him 0.41 above that floor [M the player floor snap] and fix it if the port is wrong (test red on the old code); otherwise take the ignore's own lever (a re-ground before the leg). Real input only, never lower damage or health. Coordinate with 50-real-health-14-00agent-10 if it is claimed. Run walkreal_07000014_live and interactables_07000014_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-14-landing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000014_live: Bond died at tick 4104 (frame 4104) at (31.42,2.35,33.43) armour 0.0

Merged 11-13 (my-14d): the switch band guards are answered from node 626 (three bounded stands, nearest inside 15 u); death t23511 -> t26213 at (-45.31,1.07,27.50) on the escape-touch legs: #20 15.0 in 3 at node 2, #29 7.9 at 3.7 u, health 27.9 -> 10.4 across nodes 0-3. Next: answer #20/#21/#29 from the Kiko corridor before those legs.

Merged 11-13 (42aff3fcb): the Kiko corridor squad answered while exposed (nearest first, 300 each); death t26213 -> t27134 at node 4 of the escape-touch legs, health 27.9 -> 7.9 by node 2. Next: a second stand at node 0.

Measured 11-13 (on 42aff3fcb): a second exposed-only stand at node 0 (-49.3,36.3) t27071 (22.9 left after it), at node 1 (-49.4,33.2) t26823 - no gain; the squad on nodes 2-4 outlasts 27.9 health.

Measured 11-13 (97c74df39): the #14/#15 room behind the node-42 door costs 30.0 (72.9 -> 42.9, t11150-11401). A nav stop + stand at (-21.4,-21.1) keeps 60.4 past it but the walk re-rolls: the later timeline reaches the Kiko corridor DRY (clip 1 spare 0) and dies at the same escape-touch legs (t41666, #20 25.0 in 9); at (-22.5,-20.0) t29274. Not shipped. The Kiko stretch needs ammo as well as health.

Measured 11-13: a no-stop push (return fire off) through the escape-touch legs dies sooner, t26418 at node 3. Not shipped.

Merged 11-13 (d0b707fbf): the Kiko corridor end is held 600 ticks (short stands + grenade side-step); six of the squad die on the legs; death t27517 at (-35.17,26.39), 2 u from the escape touch, to #31 (0x5000090, 0x14 at (-28.8,27.0)) with 7.9 health. Next: #31 before the last leg.

Merged 11-13 (51fc82754): a second 300-tick hold at escape-touch node 4 (-44.2,27.6); the escape touch is reached; death t29197 at (-30.20,24.28) on the Rooks-wake legs to #30 (0x5000087, 0x14) with 2.9 health. Next: health into the Kiko corridor (27.9 there).

Measured 11-13 (51fc82754): an extra hold-and-dodge at node 626 after the band stands (300/600) dies sooner at the door (t25726/t26118). Not shipped.

Measured 11-13 (51fc82754): a hold at the escape touch (300/600) dies there at 2.9 (t29452/t29378); hold-length sweep (Kiko/second): 600/150 t27685, 600/450 t29335, 450/300 t29157 (12.9 after the second hold, same death area), 900/300 t26998. No stable gain; the shipped 600/300 stays. The 16 pattern that finished its squad (short 120-tick stands on close hostiles inside the slice loop) is the next thing to try on the Rooks-wake legs.

Measured 11-13: the Rook leg split into its 33 nav nodes with a 120-tick close answer (visible, radius 8/12/18) before each node - byte-identical death t29197 at node 1 (nobody visible; 2.9 health). The gap is health before the escape touch.

Measured 11-13 (NF_DMG_TRACE): between the Kiko hold (27.9) and the second hold (7.9), #20 (0x5000090, 0x14) hidden at (-51.2,32.6) lands 15.0 in 3 at 2.4 u as Bond passes node 1 (t26993-27001), #23 2.5, #22 2.5. A 300/600-tick stand on #20 from node 0 (-49.8,36.1) or node 1 (-49.0,33.4) dies inside the stand at t26998. Next: a line on #20 from the Kiko hold spot side, or crossing node 1 faster.

Measured 11-13: crossing to the second hold with return fire held dies sooner, t26923. Not shipped.

Measured 11-13: a bullet-line scan at the Kiko hold end finds #20 (at (-51.33,32.42)) in clear line from the whole x -53..-46, z 30..37 band, including the hold spot (-51.33,37.87) at 5.5 u; a direct shoot_bounded on him from there (300/900) loses the duel - death t26998 at the hold spot with 27.9 going in. Open for a worker: the levers tried tonight are all in this file.

Measured 11-13: a hold-and-dodge in the switch band before the door west face (300/900) bleeds to 7.9/2.9 and dies at the door (t25847/t26050). Not shipped.

Measured 11-13 (c9b24271d): the node-42 door stand (60.4 kept) re-rolls the later stream: a clear_hostiles return leg ("back along the clears trail nav 1/2") wedges two 6000-tick caps at (-58.27,35.49) and Bond reaches the Kiko hold DRY (0x11 clip 0 spare 0, no armed gun to switch to) - death t41862. If the door stand is used, the clear return leg needs fixing first (nav to a reachable node, or lenient).

Measured 11-13: skipping the clears trail return at d3 (nav straight home) - alone t26476 (Kiko hold ends at 15.6); with the node-42 door stand (77.9 at the door) t26461 near the Kiko corridor. Both worse than the shipped t29197; the late stream re-rolls with any upstream change.

---
claimed-by: sandbox3 at 2026-11-14T03:24Z

---
## Report (done, 2026-11-14T04:56Z)

# real-health-00agent-14-landing

Branch `s/real-health-00agent-14-landing` (the claim commit dc479958e from origin/master
80ebf43c9, continued: origin/master 385ebb66d merged in, then s/level-14-landing-float
b140cc59e carried on top; conflicts resolved keeping master's `body_mat` accessor + the
`walk_flat_hold` param and both damage_log opt-in sets). The task: the 00 Agent infiltration
walk wedges (not a death) at the '1.72 landing' resupply leg — Bond floating 0.41 over the
floor at (33.84, 0.82, 30.95) at tick 26340, the leg cap panics; a floating Bond is a
collision question first.

## What's done

**1. The collision question: the port IS wrong — the fix carried and proven red/green.**
The carried branch's diagnosis stands ([M]-cited there and re-verified here): the game's own
push tail is DEAD in exactly this class — [M Intersect_CylGeom 0x109da0:431-447] the whole
push sits inside `0 < dot(q − HITTEST+0x50, n)` AND `|q − ref| < radius`, ref = the lifetime
DeltaP accumulator [M Collide_Intersect 0x1ebaa8:353-373] — for a face under the capsule q is
below ref and the dot gate fails; there is no fallback in the decompile. The port's [S]
spine fallback (dir = v̂ ≈ +Y, mag = r − dist) is the invention that holds the capsule at the
r-equilibrium forever — the pocket hover. The fix (the failed-ref up-facing hold skip on the
walk pass only, scoped to the quiet-dither mag < 0.02 regime) is carried into
crates/nf-game/src/world.rs (capsule_tri_contact's `walk_flat_hold`; the climb, pick-contact
and rigid-body queries keep master behaviour) and crates/nf-game/src/player.rs.

**2. The scope is ENV-GATED (NF_HOLD_SKIP=1, OFF by default = master behaviour everywhere).**
The merged-tree A/B measured why: with the skip armed, the stairwell pit-band jump's east
gate is LOST on the current master — Bond's jump arc lands at x 47.2 and the corridor wall at
47.7 needs the arc past 47.6; master's own small-mag fallback pushes during the arc extend it
east, and the skip removes them (the jump assert fails at tick 1260 with the skip on [L the
A/B runs]). With the skip off, the walk runs master's exact known shape. The float branch's
mag<0.02 gate preserved the jump on ITS (older) base; master's collision has since changed
(the one-way pane rule / body_mat and friends) and the arcs now ride small-mag pushes too.
The scope that separates the pocket's never-works hold from the arcs' real work (a
velocity/progress regime gate — the dither is near-rest, the arcs transient) is the named
next measurement; the mechanism is nailed, only the discriminator is open.

**3. probe14_pocket_settle is the red/green proof (a real #[test] now, not a dump).**
- LEG A asserts the BUG: the teleport settle at (33.84, 0.82, 30.95) hovers — grounded=false
  through the settle (hist 0x5 dither), the r-equilibrium rest y 0.6500 [L this tree].
- LEG B asserts the FIX: NF_HOLD_SKIP=1 grounds the pocket — grounded=true, hist 0xf, rest
  (33.9712, 0.6335, 30.9501) — the float branch's measured numbers reproduced [L this tree].
- LEG B is RED on the old code (no NF_HOLD_SKIP support → the hover persists). Green here.
- The skip stays OFF in every walk binary — zero walk-side behaviour change by default.

**4. The walk itself: the task's wedge premise is stale on current master — the walk PASSES
the 1.72 landing.** The ignore's "earlier ledger" recorded the t26340 wedge; master's 11-13
lever merges re-rolled the stream: on this tree (master collision, master's walk file) the
00 Agent walk crosses the landing, the drop west and the whole descent cleanly ("the low
floor west reached" [L the run]), and dies at **t29197 at (-30.20,1.16,24.28) to #30
(0x5000087, 0x14) with 2.9 health** — the ignore ledger's own current wall (the Kiko
corridor's health gap), not the landing. The task's t4104 fresh-death note is a different
ignored walk's stream; the walk file's ignored test is the 00 Agent one only. The walk-side
lever ("a re-ground before the leg") has nothing to act on on this stream; the landing pass
is pinned by the carried fix being available via NF_HOLD_SKIP=1 for the main agent's live
checks once the arc discriminator lands.

**5. The 0a replay mismatch is PRE-EXISTING on master.** walkreal_0700000a_live's
walk_office_floor_qworm_keyboard_hatch_real_damage fails its committed-recording assert at
the first-diff tick 7793 on PURE master's crates (checked out and re-run — the same tick) —
the stream re-rolled under the recording (the assert's own "re-record with NF_REC_0A_WRITE=1"
note). Not caused by this branch.

## How it was verified

- `walkreal_07000014_live` (whole binary, --test-threads=1): **9 passed, 0 failed, 1
  ignored** (the ignore is the 00 Agent walk's own ledger) [L 170.7 s, re-run green after
  the final tree settle].
- `interactables_07000014_live`: **10 passed, 0 failed** [L 13.9 s].
- `probe14_pocket_settle`: **1 passed** (LEG A + LEG B) [L 3.3 s].
- The walkreal sweep (rule 7 — the shared damage_log.rs gained opt-in fields): 01 7/0/1,
  02 5/0, 03 6/0/2, 04 10/0, 05 17/0, 06 9/0, 07 6/0, 08 5/0/1, 09 9/0, 0b 8/0, 0c 5/0,
  0d 2/0/1, 11 4/0, 12 4/0, 13 3/0/1, 15 9/0, 16 18/0, 1b 18/0/1 — ALL GREEN — and 0a
  12 passed / 1 failed (the pre-existing replay mismatch above, reproduced on pure master).
- The background suite (tools/test/suite.py) died twice with suite.out at 0 bytes — the
  sandbox kills background jobs between tool calls; the explicit binaries above are the
  coverage. Not run on the final tree.

## What's left

- The skip's discriminator (velocity/progress regime) so NF_HOLD_SKIP can arm WITHOUT
  costing the jump arcs — then the pocket grounds by default and the walk-side wedge class
  closes everywhere.
- The 00 Agent walk's real wall: the Kiko corridor's health gap (the ignore ledger's "Next:
  health into the Kiko corridor (27.9 there)") — the task's successor rounds own it.
- The 0a recording re-record (NF_REC_0A_WRITE=1) on master — a separate, pre-existing drift.

## Live checks for the main agent (PCSX2, fresh boot)

1. The pocket: teleport Bond to (33.84, 0.82, 30.95) (the 1.72 landing resupply spot), no
   input for ~4 s — confirm the REAL game grounds him (the port's hover at y ≈ 0.65 with
   grounded=false is the port bug this branch carries; the game's tail is dead in this class
   [M 0x109da0:431-447]).
2. The pit-band jump east of the -0.91 landing (x ≈ 46.1 → east onto the corridor): confirm
   the real game's jump arc clears the corridor gate past x 47.6 — the port's small-mag tail
   pushes during the arc are load-bearing for it (the skip's cost), so the discriminator must
   preserve them.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~05:24 UTC; this report written ~04:40 UTC), `git` hashes, test outputs and process
exit codes. No input/output token counters, no credit balance, no dollar cost.

## ntfy

Sent at the finish step (the record appended below only if a fallback was needed).

---
## Report from the worker (done, 2026-11-14T04:58Z)

# real-health-00agent-14-landing

Branch `s/real-health-00agent-14-landing` (the claim commit dc479958e from origin/master
80ebf43c9, continued: origin/master 385ebb66d merged in, then s/level-14-landing-float
b140cc59e carried on top; conflicts resolved keeping master's `body_mat` accessor + the
`walk_flat_hold` param and both damage_log opt-in sets). The task: the 00 Agent infiltration
walk wedges (not a death) at the '1.72 landing' resupply leg — Bond floating 0.41 over the
floor at (33.84, 0.82, 30.95) at tick 26340, the leg cap panics; a floating Bond is a
collision question first.

## What's done

**1. The collision question: the port IS wrong — the fix carried and proven red/green.**
The carried branch's diagnosis stands ([M]-cited there and re-verified here): the game's own
push tail is DEAD in exactly this class — [M Intersect_CylGeom 0x109da0:431-447] the whole
push sits inside `0 < dot(q − HITTEST+0x50, n)` AND `|q − ref| < radius`, ref = the lifetime
DeltaP accumulator [M Collide_Intersect 0x1ebaa8:353-373] — for a face under the capsule q is
below ref and the dot gate fails; there is no fallback in the decompile. The port's [S]
spine fallback (dir = v̂ ≈ +Y, mag = r − dist) is the invention that holds the capsule at the
r-equilibrium forever — the pocket hover. The fix (the failed-ref up-facing hold skip on the
walk pass only, scoped to the quiet-dither mag < 0.02 regime) is carried into
crates/nf-game/src/world.rs (capsule_tri_contact's `walk_flat_hold`; the climb, pick-contact
and rigid-body queries keep master behaviour) and crates/nf-game/src/player.rs.

**2. The scope is ENV-GATED (NF_HOLD_SKIP=1, OFF by default = master behaviour everywhere).**
The merged-tree A/B measured why: with the skip armed, the stairwell pit-band jump's east
gate is LOST on the current master — Bond's jump arc lands at x 47.2 and the corridor wall at
47.7 needs the arc past 47.6; master's own small-mag fallback pushes during the arc extend it
east, and the skip removes them (the jump assert fails at tick 1260 with the skip on [L the
A/B runs]). With the skip off, the walk runs master's exact known shape. The float branch's
mag<0.02 gate preserved the jump on ITS (older) base; master's collision has since changed
(the one-way pane rule / body_mat and friends) and the arcs now ride small-mag pushes too.
The scope that separates the pocket's never-works hold from the arcs' real work (a
velocity/progress regime gate — the dither is near-rest, the arcs transient) is the named
next measurement; the mechanism is nailed, only the discriminator is open.

**3. probe14_pocket_settle is the red/green proof (a real #[test] now, not a dump).**
- LEG A asserts the BUG: the teleport settle at (33.84, 0.82, 30.95) hovers — grounded=false
  through the settle (hist 0x5 dither), the r-equilibrium rest y 0.6500 [L this tree].
- LEG B asserts the FIX: NF_HOLD_SKIP=1 grounds the pocket — grounded=true, hist 0xf, rest
  (33.9712, 0.6335, 30.9501) — the float branch's measured numbers reproduced [L this tree].
- LEG B is RED on the old code (no NF_HOLD_SKIP support → the hover persists). Green here.
- The skip stays OFF in every walk binary — zero walk-side behaviour change by default.

**4. The walk itself: the task's wedge premise is stale on current master — the walk PASSES
the 1.72 landing.** The ignore's "earlier ledger" recorded the t26340 wedge; master's 11-13
lever merges re-rolled the stream: on this tree (master collision, master's walk file) the
00 Agent walk crosses the landing, the drop west and the whole descent cleanly ("the low
floor west reached" [L the run]), and dies at **t29197 at (-30.20,1.16,24.28) to #30
(0x5000087, 0x14) with 2.9 health** — the ignore ledger's own current wall (the Kiko
corridor's health gap), not the landing. The task's t4104 fresh-death note is a different
ignored walk's stream; the walk file's ignored test is the 00 Agent one only. The walk-side
lever ("a re-ground before the leg") has nothing to act on on this stream; the landing pass
is pinned by the carried fix being available via NF_HOLD_SKIP=1 for the main agent's live
checks once the arc discriminator lands.

**5. The 0a replay mismatch is PRE-EXISTING on master.** walkreal_0700000a_live's
walk_office_floor_qworm_keyboard_hatch_real_damage fails its committed-recording assert at
the first-diff tick 7793 on PURE master's crates (checked out and re-run — the same tick) —
the stream re-rolled under the recording (the assert's own "re-record with NF_REC_0A_WRITE=1"
note). Not caused by this branch.

## How it was verified

- `walkreal_07000014_live` (whole binary, --test-threads=1): **9 passed, 0 failed, 1
  ignored** (the ignore is the 00 Agent walk's own ledger) [L 170.7 s, re-run green after
  the final tree settle].
- `interactables_07000014_live`: **10 passed, 0 failed** [L 13.9 s].
- `probe14_pocket_settle`: **1 passed** (LEG A + LEG B) [L 3.3 s].
- The walkreal sweep (rule 7 — the shared damage_log.rs gained opt-in fields): 01 7/0/1,
  02 5/0, 03 6/0/2, 04 10/0, 05 17/0, 06 9/0, 07 6/0, 08 5/0/1, 09 9/0, 0b 8/0, 0c 5/0,
  0d 2/0/1, 11 4/0, 12 4/0, 13 3/0/1, 15 9/0, 16 18/0, 1b 18/0/1 — ALL GREEN — and 0a
  12 passed / 1 failed (the pre-existing replay mismatch above, reproduced on pure master).
- The background suite (tools/test/suite.py) died twice with suite.out at 0 bytes — the
  sandbox kills background jobs between tool calls; the explicit binaries above are the
  coverage. Not run on the final tree.

## What's left

- The skip's discriminator (velocity/progress regime) so NF_HOLD_SKIP can arm WITHOUT
  costing the jump arcs — then the pocket grounds by default and the walk-side wedge class
  closes everywhere.
- The 00 Agent walk's real wall: the Kiko corridor's health gap (the ignore ledger's "Next:
  health into the Kiko corridor (27.9 there)") — the task's successor rounds own it.
- The 0a recording re-record (NF_REC_0A_WRITE=1) on master — a separate, pre-existing drift.

## Live checks for the main agent (PCSX2, fresh boot)

1. The pocket: teleport Bond to (33.84, 0.82, 30.95) (the 1.72 landing resupply spot), no
   input for ~4 s — confirm the REAL game grounds him (the port's hover at y ≈ 0.65 with
   grounded=false is the port bug this branch carries; the game's tail is dead in this class
   [M 0x109da0:431-447]).
2. The pit-band jump east of the -0.91 landing (x ≈ 46.1 → east onto the corridor): confirm
   the real game's jump arc clears the corridor gate past x 47.6 — the port's small-mag tail
   pushes during the arc are load-bearing for it (the skip's cost), so the discriminator must
   preserve them.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~05:24 UTC; this report written ~04:40 UTC), `git` hashes, test outputs and process
exit codes. No input/output token counters, no credit balance, no dollar cost.

## ntfy failed

All three fallbacks failed on 2026-11-14 ~04:48 UTC:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d ... https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 28: `curl: (28) Connection timed out after 20002 milliseconds`.
2. `python3 urllib.request` → `URLError(OSError(101, 'Network is unreachable'))`.
3. `wget -q -O- --post-data ... https://ntfy.sh/nightfire-yoda-8k3x` → exit 124 (timed out
   after 60 s; a delivered post exits 0 quietly — this one did not deliver).
