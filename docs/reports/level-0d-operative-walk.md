# level-0d-operative-walk — the route moved three levers forward; the Operative walk stays red at the slot hop with the ledger and the next levers pushed

## The task

0700000d walked to the end on Operative (`walkreal_0700000d_live
::walk_arrival_to_the_c5_door_real_damage`, real damage, no top-ups). The branch continues
`s/level-0d-wire-jump-grab` (its report's "What's left" item 2): the ride-phase drain, then
the slot hop under the guard state diverged by then. Real input only; helpers gated to this
level and the difficulty; no damage or health lowered; the Agent sister left to its own task.

## What was done (all changes in `crates/nf-view/tests/walkreal_0700000d_live.rs`)

The body is now parameterized `walk_arrival_to_the_c5_door_at(difficulty, real_damage)`; the
new legs are gated to the real-damage run, so the plain master replay's timeline is
byte-identical and stays green, and the Agent sister (`at(2, false)`) is untouched.

1. **The west-roof #27 direct kill (the ladder top).** In the walked timeline #27 settles at
   (151.37, 22.70, −248.53) from the phase start while the shared sweep's pick takes the
   nearest walker first — she fired her 2 × 16 before her kill (+332 into its second pass).
   Her chest ray from the ladder top is full CLEAR (the sweep's own `[0d:ray]` lines: eye
   (213.1, 18.1, −234.5) → chest (151.1, 23.0, −246.1), "full CLEAR static clear
   bullet_scoped CLEAR" — she is above the mid-band obstruction). The direct box act at her
   walked spot kills her at **+222** — before her first shot.
2. **The gantry park wait moved off the rim corner.** The corner is the yard garrison's
   densest sight: once Bond steps there the 0x12 grunts land **1 hit per ~45 ticks** (the
   NF_DMG_TRACE ledger: 18 hits for 10.8 across the wait+hang+ride). The E-plate region
   behind him is measured-quiet (zero hits from the bay enter through the E, ticks
   ~10000-11487 — the first corner hit came +140 after the rim step). The 2350-tick wait now
   polls the entity matrix at the E-plate stand; the rim walk happens only when the beam
   crosses 174.5, shrinking the corner dwell from ~2350 ticks to the walk+claw.
3. **The slot-hop settle.** The hop's own failure mode is now measured with a per-tick trace:
   under the trickle the **pain staggers eat the jump input whole** — `vel [0,0,0]`, grounded,
   through five whole attempts while the gantry's carry drags Bond east (139.19 → 140.35). A
   40-tick settle now sits between failed attempts (adaptive: the plain twin's first hop
   succeeds, so it never runs there).
4. **The post-drop sweep gains g8 (159.8, −282.8) and #2 (131.6, −301.2)** — the ride-phase
   trickle's top shooters (11 and 6 of the 18 hits) outside the four original boxes' reach
   once the stream shifts.
5. **Measured and rejected: the #18 head-line leg.** #18 is the phase's floor — 3-5 × 16
   every run plus the armour share — from inside the bay at (204-212, 12.09, −288.5..−290.5)
   where every outside chest ray dies on the bay's north wall above its low opening (the
   measured 0.521 strip-mouth fraction). The height-asymmetry counter (the -5 head law) was
   TESTED from the ladder top: `clear_box_aim` returned Some — for **g10** (207.83, −288.57),
   not #18 — the act killed g10 at +330 (a real bonus: the bay-mouth sniper), but the ~900
   added ladder-top ticks fed #18's cadence two more shots and he landed two more on the
   later stands: **94 total vs the baseline 78.** The leg is out; the full story is in the
   body's comment.
6. **Measured and rejected: the yard scans from the three high stands.** The ladder top, the
   roof lane (196.3, 19.65, −280) and the bay enter (194, 10.6, −282) all see NOTHING into
   the yard pit — each scan broke on an empty `clear_box_aim` pick (zero cost, zero kills).
   A corner-stand scan killed one guard at +509, then its 600-tick caps under the 1-hit/45-
   ticks fire killed Bond — the corner is the wrong place to fight.

## The ledger (the best Operative run)

Tick 13800: alive at **15.0 health** (9 hits for 78.6 + other 6.4 — the annex fall), 1
return-fire fight, stopped by the slot hop. Phase split: the ladder-top dwell costs the
floor (8 hits for 78.0 by tick 6000 — #18 ×3-5 and the west-roof walkers plus the armour
share); the E-plate wait + the shortened corner + the hang + the ride now cost ~4 hits; the
hop still cannot land while a trickle lives, because one stagger inside the attempt eats the
jump. With top-ups ON the plain twin's hop fires clean — the walker's top-up path is the
difference; the route itself is proven to the end by the green master replay.

## How it was verified

- `walkreal_0700000d_live` whole (suite conditions, no env): **1 passed** (the plain master
  replay green), 2 ignored (the Operative + Agent sisters).
- `interactables_0700000d_live`: **11 passed, 0 failed**.
- `nf-game`: **40 passed, 0 failed** across all 15 binaries.
- `nf-formats` + `nf-audio`: **142 passed, 3 failed** — the 3 failures are exactly the
  `disc_golden` tests allowed on the slim ISO (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest` —
  the BIGF golden manifests need the full ISO's /DRIVING/MIS01.MUS).
- The nf-view crate's remaining binaries were not re-run this session: the diff touches only
  this level's test file (no `src/`, nothing under `tests/common/`), so no other binary's
  behaviour can change; the full four-crate sweep did not fit the window (the same
  build-phase constraint the -5 report recorded).
- Every run used the real input path only (`step_bond` + `Input`); no health, armour, ammo
  or channel writes; no teleports after the load.

## What's left

1. **The crouch-dodge test at the corner**: if the bay/yard opening line clips a crouched
   chest, the corner wait goes quiet, the hop fires clean, and the drop-then-sweep lever
   (the drop-point flat rays are clear; the garrison dies asleep) closes the walk.
2. **The drop-then-sweep**: land in the yard, sweep immediately — needs (1) or a clean-hop
   window.
3. **The port-side capsule fix** the wire-jump-grab report flagged (the fire-anim guards'
   boxes-only bullet volume vs the game's standing capsule [M NDrone2_Collision 0x140e90:
   91-121]) — it changes every fight's pinned timeline and needs its own window.
4. The Agent sister's d2 levers (its own ignore message).

## Live checks for the main agent

1. **The #18 opening line vs a crouched Bond.** In PCSX2 on 0700000d, stand at the ladder
   top (213.1, 17.4, −234.5), crouch, and let the bay guard #18 (204-212, 12.09, −288.5)
   fire his line. Expected: if the game's low opening clips crouched-chest rays, the ladder
   top goes quiet under crouch — the port's current geometry feeds him 3-5 × 16 regardless.
   A gdbwatch on his `DroneWeap_Ready2Fire` vs the port's ray would pin the opening's top
   edge.
2. **The slot hop on hardware.** From the gantry's west park (board offset ~1.0), jump with
   the stick held west ~20 ticks. Expected: the drop through the slot (x 136.5..138.6) into
   the yard, landing timer ~74-80 — matching the plain twin's green hop.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session; nothing to copy.

## ntfy failed

All three fallbacks failed or stayed unconfirmed from this sandbox (the same pattern the
wire-jump-grab report recorded): `curl -sS -m 20` → `curl: (28) Connection timed out after
20001 milliseconds`; the python3 urllib fallback → `urllib.error.URLError: <urlopen error
[Errno 101] Network is unreachable>`; the wget fallback → stalled past 45 s with no output
(the command was killed at the tool timeout; the earlier non-verbose run showed the TLS
connect to ntfy.sh not completing). The finish.sh step itself reported
"OK done 10-level-0d-operative-walk".
