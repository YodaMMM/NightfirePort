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
