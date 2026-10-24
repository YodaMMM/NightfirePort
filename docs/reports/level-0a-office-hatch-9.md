# level-0a-office-hatch-9 — the hall glide, the corner, the office walk (round report)

## What was asked

0700000a: dump the doorway base's triangles (x −8.5..−8.0) and the corner's at (−17.87, 4.87);
log the port's per-tick capsule contacts and pushes there; compare with the game's rules — the
slope-blind capsule push added every tick [M `Intersect_CylGeom` 0x109da0,
`Player_CollisionHandler` 0x1949a8 disasm 0x194c60-0x194c70], the feet band [M
`Player_FeetOnPoint` 0x194188], the head-on cull [M 0x109da0:302-322] — name the step that
differs, fix it, and un-ignore `the_west_hall_glide_height_matches_the_game`,
`the_corner_crosses_like_the_game`, `walk_office_floor_qworm_keyboard_hatch` as they go green.

## What I did

**Setup:** reused the sandbox's leftover checkout (work/, the slim ISO, the ALSA prefix, the warm
cargo target), set the token remote, fetched, branched `s/level-0a-office-hatch-9` from
origin/master (restoring `docs/WORKLOG.md` from the hatch-8 branch — the batch merges drop it),
pushed the branch with the WORKLOG entry at 19:31 UTC, and started the fast suite in the
background at 19:35 (`setsid`; plain `nohup` jobs die between tool calls here, as hatch-8
already found).

**The tri dumps [D port world tris]:** doorway box (x −8.9..−8.0, z 15.9..17.1): 18 tris — the
x=−8.304 wall band (y 0..2.2, n=(1,0,0), z 15.376..16.279), the z=16.279 wall plane (x
−8.604..−8.304, n=(0,0,1), y 0..2.2 with the small y 0..0.19 corner pieces at x −8.604..−8.667),
and the y=0 floor lip. Corner box (x −18.2..−17.2, z 4.2..5.1): 4 tris — the step block's west
faces (x −17.498..−17.361, y 0.734..0.838, n=(−0.99,0,0.085)/(−0.93,0,0.356)) and the y=0 floor.
The doorway "passage" between the z=16.279 plane and the north wall at z≈16.83 is ≈0.55 wide —
NARROWER than the player capsule's 1.1 diameter.

**The per-tick contact log** (new `NF_CAPS_LOG=1` diagnostic in `capsule_tri_contact`): the
glide test's walk wedges SOUTH-WEST of the doorway — 27,404 logged pushes dominated by the
x=−8.604 jamb face (8,038), the x=−8.660 column (11,912 over four tris) and the z=16.279 plane
(4,748) — Bond pressed into the jamb pocket, STUCK at (−9.15, 1.09, 15.92) after 6000 ticks.

**Named step that differs (measured, not yet fixed):** the game's push does not fire when the
capsule AXIS OVERLAPS a wall plane. [L fresh boot] the game glides the z 16.29 line with its
capsule 0.011 inside the z=16.279 plane — no north shove, glide 1.0891 → peak 1.18 → settle
1.1147. The port's ungated contact fires **+0.539/tick** at that overlap (the probe trace: from
(−7.0, 16.29) the walk is launched to z 16.8288 and pinned on the north wall; the raw capsule
then crosses the doorway cleanly at 1.0891 — no climb, because the brush never happens at the
game's line). The dead-on raw hold ALSO crosses the corner cleanly (27 ticks, y 1.130 → 1.1191
against the game's 1.127 → 1.120 [L]) — the corner test's wedge is the same overlap-push shape
pressing differently on the walked approach. So: the slope-blind per-tick push [M 0x1949a8:136-140]
is ported and correct; the feet band [M 0x194188:106] is ported ([L]-bit-exact spawn rest); the
head-on cull [M 0x109da0:302-322] is implemented but OFF (`HEAD_ON_CULL_ON = false`) — and is
NOT the differing step here: it only touches |n·axis| > 0.999 faces (floors/ceilings), and both
wedging wall sets are vertical. The differing step is the missing **overlap gate** on the push
tail.

**The gate candidate, tested and refuted in detail:** the C decomp gates the whole push on
`0 < dot(hitpoint − HITTEST+0x50, plane)` [M 0x109da0:422-431] — I implemented
`dot(q − B, n) > 0` behind `NF_CAPS_GATE=1` (B = the spine's bottom end). With it the corner
test CROSSES (the first configuration that does — the wedge is gone, ~27 ticks) but misses the
exit-y pin both ways: 1.0655 (floors gated too — he sinks) and 1.2142 (floors exempted — a
below-B lift survives), against the game's 1.120 ± 0.01 [L]; and the glide walk then floats
ungrounded at (−8.05, 1.13, 15.03). The predicate's reference vector is the blocker:
HITTEST+0x50 (BL+0x790) is never written per tick anywhere in the decomp — `Collide_Update`
0x1ea398:50 zeroes only +0x10 (the push accumulator, per tick [M]), +0x50 is moved only by the
DeltaP adds [M `Collide_Intersect` 0x1ebaa8:353-373] — so its boot value and per-tick identity
(the spine bottom? the scan origin? something else) cannot be pinned from the C. The gate stays
env-gated OFF; nothing unverified entered pass/fail.

## What's left

- Pin HITTEST+0x50's live value/identity (the live checks below), re-derive the overlap gate,
  and re-test both walk tests + the office walk (`walk_office_floor_qworm_keyboard_hatch`, whose
  ai-lane mouth wedge plausibly shares the same overlap shape — not reached this round).
- The three tests stay `#[ignore]` — none is green under a defensible configuration. The
  decompile-backed assertions were not weakened.
- The full fast-suite re-run on the final tree.

## Live checks for the main agent

Fresh boot of 0700000a (a save-state resume skips the boot path):

1. **HITTEST+0x50's identity** — gdbwatch the reads of BLData+0x790 (the player's BLData is
   0x16cf0f0 at the 05 start [L]; on 0a read obj+0xE0 first) during the doorway walk
   (x −8.0..−8.6 at z ≈ 16.29): access type = read (the push tail 0x109da0:422 area), expected =
   whatever the vector holds while the capsule overlaps the z=16.279 plane — if it tracks the
   spine's bottom end, the overlap push would be gated off and the game's 16.29 line is
   explained. Record 8 bytes ×2 (the vector) + the obj+0x30 y/z for correlation.
2. **The overlap contact's push landing** — with Bond pressed at the doorway (the fresh-boot
   brush at x −8.304), gdbwatch the write to obj+0x30 from 0x1949a8:136-140 (disasm
   0x194c60-0x194c70): does the landed DeltaP stay ~0 while the capsule overlaps the wall plane
   (the gate reading), or does a z-push land and get cancelled elsewhere?
3. **The glide climb's contact set** — fieldprobe the root y across the doorway brush window:
   the [L] pins are peak 1.18 and settle 1.1147; the port's raw capsule gives 1.0891 flat when
   it misses the brush. One y-trace over ~60 ticks decides which triangles the climb actually
   comes from (the band edge at y 0.162 vs the corner pieces).

## Run report

- `cargo test --offline --release -p nf-game -- --test-threads=1` — 88 binaries, 0 FAILED.
- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live -- --test-threads=1`
  — 4 passed, 3 ignored (the task's three; the spawn-settle, the bridge-edge, west-doorway-repro
  green).
- The fast suite (`tools/test/suite.py`) started 19:35 on the pre-diagnostics tree; at report
  time it is mid-run: the 4 known slim-ISO baseline fails (identity_disc + 3 disc_golden) plus
  4 `exit -9` CRASHes on the mp/net binaries and 3 pylon-render FAILs while my concurrent cargo
  runs held RAM — treat those as suspect until the clean re-run; no walkreal/collision binary
  failed. The final-tree re-run did not fit the window (the build alone took 961 s).

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
