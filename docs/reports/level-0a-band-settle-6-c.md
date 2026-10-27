# level-0a-band-settle-6-c — the 0a radial tail restricted (the 21-test y-settle break)

Branch `s/level-0a-band-settle-6-c` (from `origin/s/level-0a-band-settle-6-b` + `origin/master`).

## Verdict

The settle-6-b radial tail (D1/D3/D4/D5 landed unconditionally in `capsule_tri_contact`) is the
21-test break, and **none of its four deltas survives measurement against the full suite**. The
landed state reverts all four to the settle-3 merged tail shape; the D2 flags-99 skip stays
env-gated OFF (`NF_CAPS_D2`), exactly as 6-b held it. The only functional difference left against
`origin/master`'s `world.rs` is the default-OFF `CAPS_D2` env gate (verified by diff: code lines
other than comments = the gate alone).

## The repro (what "Bond settles ~1 unit high" is)

`interactables_07000015_live::the_dead_ladder_never_grabs_at_point_blank` teleports Bond to
(85.2, 40.5, −2.4) over the south walkway (floor tris ≈ 40.35 [D]) and asserts the settle lands
y ∈ (40.2, 40.7). On the 6-b tail it settled **41.31509** — the spine bottom B pushed to exactly
r = 0.55 above the walkway face (equilibrium `d = 0.5500 push = (0,0,0)` in the NF_CAPS_LOG tail;
B = pos − 0.4847 [M Player_Collision 0x194df0's B arm]). With the walk spine's floor clearance
only 0.604 on the walked line, any face the tail pushes along repeats every tick and climbs to
that equilibrium — which is why floor touch plates, walk-over pickups and use-key doors fail
across 21 tests in 20 files.

## Per-delta measurement (each on its own build, NF_CAPS_LOG)

- **master tail** (the settle-3 shape): the tick-2 contact — the walkway's north-wall face with
  the spine 0.0013 BEHIND its plane — pushes THROUGH (v = p−q = −n·0.0013, dir = v̂, mag =
  0.5487), carries Bond out of the wedge, whole 15-file 12/0.
- **+D4** (the always-on dot gate `dot(v,n) ≤ 0` skip, [M 0x10a568]): that behind-plane face goes
  inert — which IS the game's flags-99 rule [M Dist2Tri 0x109770 + caller 0x10a33c] — but Bond
  stays wedged and the sloped skirt faces (n=(0.114,0.957,0.267), n=(0,0.964,0.266)) then push
  v̂·(r−|v|) every tick: the climb. This is the primary break.
- **+D3 alone** (the on-edge Q2 re-run [M 0x10a344-0x10a4fc] + the second call's flags-99 gate
  [M 0x10a510/0x10a528]): still 8/12 red on the 15-file — the re-run's q = closest_on_triangle(p2)
  changes the tick-1 edge contact's push by ~2e-4 (x −0.0005 vs master's −0.0003), which alone
  diverts the settle into the same wedge. The 6-b bisect's 01-plate-walk fix is real [M] but only
  holds in isolation.
- **D1/D5** (the always-on v̂ tail; the |v|==0 zero quad [M 0x10a5bc]) shift the same settle.

## Why the "always-on" reading over-applies (the task's gate question)

In `Intersect_CylGeom`'s decompile the whole push tail sits inside `if ((*(byte *)(HITDATA+0x51)
& 6) == 0)` [M 0x109da0:421], where HITDATA+0x51 accumulates the caller mask OR'd at
Collide_Intersect 0x1ebaa8:178 — the walk body's mask word is HITTEST+0x9a = 0x0000 [L
ee_start_05.bin BL 0x16cf0f0], so the gate does not block the walk; the gate that DOES bound the
tail's operands is the accumulator-relative arm: v = hitpoint − HITTEST+0x50 behind
`0 < dot(v, n)` [M 0x109da0:421-431] and `|v| < radius` [M :447]. HITTEST+0x50 is the lifetime
DeltaP accumulator [L boots (0,0,0); drift ≈15 m from its owner mid-play], so on the game's real
walked lines that arm near-never fires — the game's walk vertical is the feet band
[Player_FeetOnPoint 0x194188: the ray down `fVar11+0.6`, grounded at hitdist ≤ fVar11+0.1] plus
the walked dy, not this tail. The port's substitute tail (dir = v̂, mag = r−|v| unconditionally)
is the thing the 6-b deltas sharpened; sharpening it exposed the substitute in 20 files. The
restriction therefore keeps master's substitute exactly.

## Verified

Suite runs via `tools/test/suite.py` (foreground batches — this sandbox kills background
processes between tool calls, so the single nohup run could not survive; totals combined):

- batch 1 (nf-view core of the failure list: walkreal_0700000a/01/08/0b/11,
  interactables_0700000d/11/15/16): **60 passed, 0 failed, 3 ignored** (4 min)
- batch 2 (fix_08_collision, fix_0c_hatch, fix_mission, fix_searchlight_remainder, gt_los,
  gun_turrets, interact_07000006, interact_0700000c, level_end, mp_bot_armour_online, mp_perf_16,
  mp_world_sync, probe_07_route): **56 passed, 0 failed, 0 ignored** (1 min)
- batch 3 (walkreal_07000002..07/09/0c/10/12/14 spread): **29 passed, 0 failed, 1 ignored** (8 min)

Combined: **145 passed, 0 failed, 4 ignored** across 30 test binaries — every file named in the
task's 21-test list is green, and the 0a walk battery keeps its 5/0+3i shape (the trio's ignores
are unchanged from 6-b). The full 856-binary suite did not fit the session window (see below);
the batches cover the entire known-broken set plus a walkreal spread. nf-game/nf-formats/nf-audio
crates were built; their binaries were not re-run this round (unchanged by this branch except
world.rs, which batch 1-3 exercise through nf-view).

## What's left (for the next round)

1. The real walk-vertical mechanism: port the feet band + walked dy as the y-settle authority
   ([M Player_FeetOnPoint 0x194188] + Player_Collision's dy integration), then re-land the tail
   deltas (D2's flags-99 skip is [M]-faithful and wanted) without the substitute carrying the
   vertical. Until then the substitute tail must stay byte-master.
2. The tail operand decode at [M 0x10a53c-0x10a61c] — the decompile reads the tail's v as
   hitpoint − HITTEST+0x50 while the 6-b disassembly read v = query − Q; the EE COP2 stores around
   those addresses need a careful metal-level re-read (my capstone dump is in scripts/tail.dis,
   polluted by undecoded MMI/COP2 words).
3. The full 856-binary suite run on this tree (the next session can start it early — in this
   sandbox it must run in foreground batches or inside one tool call).

## Live checks for the main agent

1. **The behind-plane wall face** (07000015 walkway): fresh boot, walk Bond south into the
   walkway's north wall at (85.2, ~40.5, −2.4) pressing south. Watch BL+0x760/0x770 (the walk
   spine A/B) and BL+0x790 (the accumulator): does Bond's y stay ≈ 40.4-40.5 (no climb), and does
   the accumulator move only ~0.1/tick horizontally? Expect: no upward ratchet.
2. **The tail's gate byte**: at the same wall contact, break on Intersect_CylGeom 0x10a568 and
   read the operand registers — is the dot test `dot(hitpoint − HITTEST+0x50, plane)` (the
   accumulator-relative arm) or `dot(query − Q, plane)`? This settles what's left vs what's
   decoded ([M 0x10a53c-0x10a61c] vs the decompile 0x109da0:421-431).
3. **Dist2Tri #2 flags**: at an on-edge contact (the walkway's skirt, q on the tri edge), confirm
   the re-run at 0x10a4fc writes copy+0x50 only when the projected query is in front of the plane,
   and that the caller's 0x10a510/0x10a528 gate then sees flags 99 otherwise (the face inert).

## Usage

Token/credit/context figures: not visible to me (no token or cost display is exposed in this
environment; the sandbox shows only wall-clock time, checked with `date -u` at 06:02 UTC start).
