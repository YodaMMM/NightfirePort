# level-0b-waiting-room-doors — the report

Task: level-0b-teleports-to-walks kept one movement stand-in in `walkreal_0700000b_live` — the
waiting-room exit (lift landing → door #3 (13.71, 46.78, 19.44) → the east room → door #0
(13.82, 46.78, 7.09) → the ring's east corridor). The doors E-opened (state 0x2 → 0x4) but the
walker wedged short. Find the doors' open end value and the open-leaf pose vs the game's law, fix
the port, walk the transit, delete the write.

## The root cause and the fix ([M] code)

**The port's door pick radius was wrong.** `Doors::collide_step` picked contacts with
`r = 1.5·r_door + r_pick` (2.23 for these doors). The game's law
([M] `Collide_PickObj` 0x1eae80:100-110) is the door's own `obj+0x8C` radius ALONE: the sphere
centre+radius `obj+0x80..0x8c` are copied onto the hittest, the closest point on the A..B segment
is taken (`vecutil_calculate_closest_point_on_line`), and the pick fires only while
`SqDist3D(centre, closest) < r_door²` — `r_door² <= d²` returns. No 1.5 factor, no pick radius.

With the inflated radius, Bond standing ~1.2 m from the hinge (where the E use-sphere puts him)
was a PERMANENT contact, and `Door_ReactToHit`'s anti-crush ([M] 0x135980:85-97: while OPENING,
`f = local.z·(+0x5C) > 0.0002` → reverse to CLOSING) reversed the swing every ~3 ticks. The
`NF_DOORTRACE` per-tick log (added to `door.rs`, env-gated) caught the oscillation:
`pos01 0.3966→0.3793→0.3621 (OPENING) → reversed 0.3448→0.3621→0.3793 (CLOSING) → reversed…`
forever — the leaf never cleared the doorway. That is the stand-in's whole cause.

**The fix** (`crates/nf-game/src/door.rs`, `collide_step`): `let r = self.list[i].sph.1;` — the
door's own radius, cited inline.

## The doors' end values and the pose law (what the task asked to establish)

- Both doors are disc swing doors (kind 0, flags 0x3 spline, 2 frames = 3 keys [D the 0x19 anim
  keys dumped by `probe_0b_doors`]): door #0's keys rotate 180° → 90° → 0°, door #3's 225° →
  135° → 45° — the CLOSED pose is the MIDDLE key (the leaf across the doorway: #0 spans
  z 5.86..7.10 at x ≈ 13.8; #3 spans the diagonal (13.71, 19.44)→(12.84, 18.54)), and the OPEN
  ends are keys 0 and 2 — the leaf swung fully aside (door #0 open = the leaf along ±X, its thin
  in-plane edge at z 6.97..7.21; door #3 open = the leaf along the mirrored diagonal).
- The open END VALUE depends on the approach side: [M] `Door_Activate` 0x135be0:136-146 —
  `dir = sign((inv_closed·activator).z)·(+0x5C)` → the leaf swings AWAY; opening runs pos01
  0.5 → 1.0 or → 0.0 (both legal ends, clamped [M] `Door_Interp` 0x135738:33-61). Measured with
  the fix: door #3 from the waiting-room side opens to **pos01 1.000** (state 4), door #0 from
  the east-room side opens to **pos01 0.000** (state 4) — the away-side end in both cases.
- `+0x5C` is not a disc param: [M] `Door_Create` 0x136100 never stores it — `Door_SetupSwing`
  (0x135440:65-80) derives it: `+0x5C = (Mat_GetDir(pose_t0)·Mat_GetNorm(pose_mid) < 0) ? +1 : −1`
  with [M] `Mat_GetDir` 0x1e3398 = row 2 and `Mat_GetNorm` 0x1e33d8 = row 0. The port's create
  derives the same sign the same way (`dot3(row(open,2), row(closed,0))`) — the port's swing sign
  and open-leaf pose law MATCH the game; only the pick radius was wrong.

`probe_0b_doors.rs` (committed, `--ignored`, assertion-free) dumps both doors' keys, the leaf
shape's world AABB at t 0/0.25/0.5/0.75/1.0, the doorway crossings before/after an E, the dynamic
set near each doorway, and the swing progression per 6 ticks.

## What's done / not done in the walk

- The walked-transit rewrite was built and iterated on the branch (commit f3873e62): the pit-gap
  pin + the nav lane (n41/n54/n52/n50/n49/n19/n18/n17) + both `use_door` calls. It is NOT green
  yet: the approach from the cab step-off's landing still wedges on the waiting room's south
  lanes (the pre-existing blockers the old teleport also covered: the boardroom furniture row
  x 4-6 z 20-22 and the z 16-17 wall band at x 8-14 [D the prior probe]). The test file is
  RESTORED to its green form on this branch's head; the transit rewrite lives in the branch
  history (f3873e62) for the next message to resume.
- What the iteration ESTABLISHED (all in the probe/leg evidence):
  - The cab step-off lands in the shaft-mouth pit (the 45.4 ledge, x 5-7 z 23-25 [D floor
    grid]); its rim is walled at body height on every probed line except the south-west
    ((7.5,24.9)→(5.0,23.5) clear; →(10.5,22.5)/(9.2,21.3)/(8.0,22.0)/(6.0,21.5)/(10.0,25.5) all
    hit [D]); the rim floors at (5-6, 22) read 47.3 vs the pit 45.4.
  - The east room's lane WALKS on the nav nodes (the exec-floor nav probe's legs 0.49→50→51→52→
    53→54→55 all true arrivals, d ≈ 0.93) and the KEY legs "door3→east room" (194 ticks, true
    arrival) and "door0→ring n11" (54 ticks, true) — both doorways pass once the leaves open
    (with the pick fix).
  - The game's own nav lane for the whole transit: n41 (2.91, 20.91) → n55 (9.18, 21.33) → n54
    (12.21, 19.68) → n53 (14.13, 18.35) → n52 (15.85, 16.77) → n50 (17.10, 14.00) → n49 (17.03,
    10.83) → n19 (16.23, 8.07) → n18 (14.50, 6.54) → n17 (13.18, 6.63) → n11 (10.21, 8.20) [D
    nav path 0 'Navigation'] — the n41→n55 hop is the one leg the walker cannot reproduce
    straight (the furniture row); it needs a bent lane or the game's own AI steering law.

## Verification (what I ran)

- `walkreal_0700000b_live`: **3 passed, 0 failed** (38.14 s) — on the fixed tree, the green form.
- `interactables_0700000b_live`: **9 passed, 0 failed** (18.30 s).
- Every `*door*` binary (15): **35 passed, 0 failed, 2 ignored** — 0700000d_gunimp_door 3/0,
  cutscene_door_07000014 3/0, door_aim_ray 2/0, door_glist0_e_dead 2/0, door_glist_census 1/0/1,
  door_guard 5/0, door_guards_hold_07000014 1/0, door_guards_locked_door_07000014 2/0/1,
  door_hint_boot_count 2/0, door_portal_links 3/0, door_portal 2/0, door_pose_probe 2/0,
  door_research_07000001 2/0, door_text_labels 4/0, door_wait_07000014 1/0.
- `probe_0b_doors` (diagnostic): runs green; its dump is the evidence above.
- Full suite: this sandbox kills background jobs between tool calls (the fast runner cannot
  survive; the prior two sessions hit the same wall and ran per-binary foreground batches). The
  door family + the two 0b binaries above are the touched set; the broad suite is the next
  message's first runner window. The pick-radius change touches every door on every level — the
  15-binary door family plus walkreal/interactables is the honest touched-binary coverage this
  window could run.

## Live checks for the main agent

1. Door #3/#0 open-leaf passage, fresh boot 0700000b: Bond at the exec-floor landing, E door #3,
   walk through south; E door #0, walk west to (10.21, 8.20). Record each door's open pos01 end
   and the leaf's world pose when fully open, and whether the passage passes. Expected: the
   disc's open leaf clears the doorway (the walkthrough's doors open and pass); the port's open
   poses now match [D the keys; M the laws above] — if the disc still blocks where the port
   passes, the doorway's static cels are the next gap.
2. The shaft-mouth pit exit: at the cab's top docking, walk east off the cab onto the lower
   ledge (x 5-7, z 23-25) and record how the game's player leaves it (the south-west rim line,
   a step-up/jump onto the 47.3 rim, or a lower lip west of x 5). This settles the pit-gap
   stand-in the next message will replace with a real walk.

## What's left

- The walked-transit rewrite to green (resume from commit f3873e62's lane, fix the n41→n55 hop
  around the furniture row, re-place only the pit gap or walk it after live check 2).
- The full suite over the fixed tree (the pick radius touches every door).

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session.
