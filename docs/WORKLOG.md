# WORKLOG

## level-0a-corner-walk-speed (branch s/level-0a-corner-walk-speed)

Start: 2026-10-30 10:28 UTC, branch from origin/master 06b95819.

Task: un-ignore `walkreal_0700000a_live::the_corner_crosses_like_the_game` — the 0a corner only
crosses with the below-B kill (NF_WALK_KILL=1) in 870 ticks vs the game's ~60 frames
[L 2026-10-26], riding spine-direction pushes. Find what the game does at that corner
[M Collide_Intersect 0x1ebaa8 / Intersect_CylGeom 0x109da0 / the player push-out; D the corner's
geometry], fix the port side so the corner crosses at walk speed without the kill switch, and
un-ignore the test. Every test in the file green, plus the other walkreal binaries that can run.

Outcome 11:50 UTC: the corner's mechanism identified and measured, the flip not landed in the
window. [M] The decoded cyl push tail v = p − HITTEST+0x50 (the aimed spine point minus the ref;
gates dot(v,n) > 0 ∧ |v| < radius; the edge re-aim targets the spine point closest to the REF —
the port re-aims closest to q, a divergence) cannot sustain the walk's wall response: p is
spine-clamped, so for a vertical walk spine any on-spine ref gives a purely vertical v whose
dot(v,n) gate kills vertical-wall pushes; the ELF-wide store sweep (all store opcodes, disp
0x790/0x7a0, whole image) finds no writer of HITTEST+0x50/0x60 beyond the add-back
0x1ebaa8:353-373, and [L ee_snapshot] the live value drifts ~15 m. DealWithObjHit 0x194458,
Player_Move 0x191dd0, the handler 0x1949a8 and Player_Collision 0x194df0 write no wall response.
The walk's response is the MOVE CLIP: with NF_WALK_CLIP=1 (the built, env-gated model) the corner
CROSSES at +76 ticks vs the game's ~60 frames [L 2026-10-26; measured this session], no kill
switch — and the clip's spine-plane rest matches the game's ~0.011-inside-the-plane glide rest
[L fresh boot]. Not landed: under the flag the_pick_ref_accumulator_tracks_the_game_ref,
walk_office_floor_qworm_keyboard_hatch and walk_office_floor_to_the_security_keyboard go red
(master 7 passed / 0 failed / 2 ignored), and the corner's exit-y reads 1.0891 vs the settled
1.1159 — the 0.0268 gap is the west-hall leg's unported +0.0256 climb settle (the glide sister's
mechanism). The corner test stays ignored with the new state pinned in its note. Report:
docs/reports/level-0a-corner-walk-speed.md.

## level-0a-walk-clip-flip (branch s/level-0a-walk-clip-flip)

Start: 2026-10-30 12:03 UTC, branch from origin/master dca2f368.

Task: make the move-clip the walk's wall response — reconcile the three tests that go red under
NF_WALK_CLIP=1 (the_pick_ref_accumulator_tracks_the_game_ref, walk_office_floor_qworm_keyboard_hatch,
walk_office_floor_to_the_security_keyboard; fix the port or the test, with cites), make the clip the
default (remove the env gate), and run every walkreal_* binary plus the collision/player crate tests;
report any walk that changes with the leg and the position delta.

Outcome 13:36 UTC: the clip's mechanism corrected and two of the three flip-red tests reconciled; the flip not landed. The face-plane clip (master's env-gated model) never engages on a wall press — the press never crosses the face plane within r — and Bond walked THROUGH the west end wall (x −48, measured). The game's own press response is the r-skin: [L 2026-10-26] the spine rests face+0.45/+0.55 (the pre/post phases of one retraction) and HITTEST+0x50 grows +0.1/contact-tick — capsule_tri_clip now retracts the move's approach along the contact radial v̂=(p−q)/|p−q| to dist r (tangential carries; the corner slide preserved). Root cause of the dead clip: pre_move_pos was captured AFTER the walk's matrix·(vs,vf) x/z add, so move_v was (0, gravity, 0) every walked tick — the capture now reads physics_tick's own prev (the pre-move pos); the accumulator feed is unconditional (the clip's retraction IS the applied displacement the [M] add-back 0x1ebaa8:353-373 accumulates). walkreal_0700000a_live under NF_WALK_CLIP=1: 6 passed / 1 failed / 2 ignored (was 4/3/2) — the_pick_ref_accumulator_tracks_the_game_ref and walk_office_floor_to_the_security_keyboard GREEN, walk_office_floor_qworm_keyboard_hatch still red (the ring's alarm-runner watch missed during the south corridor; the jink-4 door-leaf wedge was fixed by a route nudge, the alarm beat the walk). Default (no env) unchanged: 7/0/2. nf-game lib: 588/0/5. Left: the qworm ring reconciliation, the flip (remove the env gate), the whole-battery run. Report: docs/reports/level-0a-walk-clip-flip.md.

## level-0a-walk-clip-flip-2 (branch s/level-0a-walk-clip-flip-2)

Start: 2026-10-30 12:50 UTC, branch from origin/s/level-0a-walk-clip-flip eac2888a (master dca2f368 is its base).

Task: continue the move-clip flip — fix the qworm ring alarm leg under NF_WALK_CLIP=1 (port or test,
with cites), then flip the default (walk_clip() -> true, remove NF_WALK_CLIP and the dead push-walk
branch), run every walkreal_* binary plus nf-game's collision/player tests, quote totals, list walks
whose route changed.
