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
