# level-0a-corner-push-model-2 — the corner without breaking every other walk

Branch `s/level-0a-corner-push-model-2` = `origin/s/level-0a-corner-push-model` merged with
`origin/master` (clean). Status: **blocked** — the corner crossing is not re-landed on the
merged tree, and the measured separator classes overlap. Everything below is measured this
session on this exact tree.

## What the session established (all traced, addresses cited)

1. **The game's only displacement is the guarded REF-relative tail** — `Intersect_CylGeom`
   0x109da0:419-457 (v = p − REF gated dot(v,n) > 0, |v| < radius, accumulated into _DeltaP and
   the re-aimed probe slots). The head-on block I had earlier read as an unguarded "re-push"
   sets a FLAG (`*(ushort*)(HITTEST+0xc) |= 1`), NOT a push [M 0x109da0:482-518, decompile
   lines 483-518 — the `0.7 < dot` branch's only action is the flag write]. With the
   accumulator booting (0,0,0) [L ee_start_0700000a] and drifting ~15 m behind the owner
   [L ee_snapshot], the tail near-never arms on a fresh walked line — for ANY contact class.
   The game's wall rest (z 15.5904 [L 2026-10-30]) therefore still comes from an unidentified
   mechanism (the prior report's live check 1 stands).
2. **The corner's wedge membership changed on the merged tree.** The prior branch's +76
   crossing does NOT reproduce here: the probe (`walkreal_probe0a_corner::probe_0a_corner_walk`)
   freezes at (−17.874, 4.872) with the below-B skip active. Contact capture (NF_CAPS_LOG=1):
   the fatal sum is now the counter slab's strips (one face-interior (−0.0401, 0, +0.0153),
   one edge (−0.0383, 0, +0.0365)) PLUS the door #4 leaf's mid-height edge contacts (deep,
   r − d up to 0.078) — the leaf rides at the DISC's half-open state (state 0x2 pos01 0.50
   from the first probe leg [D the load state]) and again closed (state 0x4 pos01 0.00) after
   the walker's stuck mitigation opens/closes it. The below-B member the prior branch dropped
   is not the binding constraint anymore.
3. **Depth does not separate the classes.** Measured bands: the corner slot's edge contacts
   r − d 0.016..0.078; the Q-Worm jink's cap contacts r − d 0.0505..0.092 — the bands OVERLAP
   (the slot's leaf-mid contact sits at 0.075-0.078, deeper than the jink's shallowest).
   A threshold rule (0.05) was built and measured: it restores the jink and re-wedges the
   corner through the deep leaf-mid member.
4. **Face-class splits measured**: the leaf/counter faces are near-vertical (|n[1]| ≈ 0.000-
   0.005); the jink's cap mixes up-facing (n[1] 0.918/0.934), oblique (0.485-0.518) and one
   vertical (0.017). A |n[1]| < 0.3 gate keeps the jink's obliques but still broke it (the
   vertical member is load-bearing there); a ≤ 0.7 gate broke the jink too.
5. **The dynamic-source skip (landed on the branch)**: the port's collision walk separates the
   static cel triangles (`stat`) from the moved-object triangles (`dynamic.geo`) at the call
   site. The landed rule drops the walk-path EDGE-contact fallback ONLY for the dynamic class
   (`capsule_tri_contact`'s new `dynamic_obj` arg — the [M] shape: the moved leaf's edge
   contacts push nothing in the game, the REF tail being inert). Measured:
   - `walkreal_0700000a_live`: **7 passed / 0 failed / 2 ignored** (the Q-Worm jink green —
     its cap contacts are static).
   - The corner: the leaf contacts drop (the capture shows only the counter's pushes
     surviving, sum (−0.078, 0, +0.052) — non-cancelling) BUT a NEW freeze appears at the
     same spot with Bond's capsule bottom climbing ~0.10 (b[1] 0.602 → 0.699 across the
     capture rows) while x/z freeze. The residual mechanism is NOT the leaf-edge pushes and
     is unidentified — the next round's first question.
6. **The dynamic class is too broad for the 40-red list.** With the landed rule:
   `ladder_02_live` 1/2/0, `fix_08_collision_live` 3/1/0, `interactables_07000002_live`
   15/1/0 (FAILED — the same tests the prior branch's below-B skip broke), while
   `walk_0700000a_live` 3/0/0 and `pickups08_live` 5/0/0 pass. The ladders' rails and the
   08 collision's load-bearing edges flow through the SAME `dynamic.geo` class as the door
   leaf — the separator must go one level deeper: per-object source (the door leaf vs the
   ladder rails vs the 08 movers), plumbable as a source byte on the dynamic geo entries
   (the entries already carry `mats`).

## What's left (the next round's plan)

1. Per-object source tag on `dynamic.geo` entries → skip the walk-path edge fallback ONLY
   for door-leaf geometry (the game's leaf: a MOVED DOOR object — the leaf's contacts are the
   [M]-phantom class; ladders/movers keep the stand-in). Verify: the corner (the residual
   y-climb freeze question first), then the 40-red list.
2. The residual corner freeze (5 above): identify the y-climb writer (the walk's
   step/ground logic, not the capsule push — the capture shows the climb WITHOUT any +y push
   surviving).
3. The un-ignored corner test still pins the exit y (1.0891 vs the game's 1.1159 [L
   2026-10-26]) — the climb-settle handoff (level-0a-hall-glide-settle) stands.

## How it was verified

- `walkreal_0700000a_live` (the landed tree): 7/0/2 — quoted total.
- `walkreal_probe0a_corner` (the landed tree): 3/0/0; the probe's crossing line: the 400-tick
  cap, wedged (−17.874, 4.872).
- The 40-red samples on the landed tree: ladder_02 1/2/0, fix_08_collision 3/1/0,
  interactables_07000002 15/1/0, walk_0700000a 3/0/0, pickups08 5/0/0.
- The full walkreal_* set and the full suite: NOT run — the background suite runs were killed
  by the sandbox between tool calls (three attempts), and the foreground window went to the
  class measurements above. The next round must run them before any push to master.

## Live checks for the main agent

1. The prior report's checks 1-3 stand (the +0x50 accumulator, the counter press, the E-less
   door open).
2. NEW: with Bond pressed into the corner (−17.87, 4.87), watch the capsule bottom's y
   (BL+0x790's B arm) — the port climbs 0.10 there with NO surviving +y capsule push; which
   game system lifts/writes the walk's y under a squeeze? (The walk's step/ground band, not
   the push tail.)
3. NEW: the door #4 leaf — its collision triangles' object class in the game's collide pass
   (the leaf's obj chain vs the static cels): does the game's collide pass even include a
   half-open leaf's face column in the WALK query's candidate set (the straddled-cel scope vs
   the object's own chain)?

## Usage

not visible to me (no token/credit/context figures are exposed to this session).
