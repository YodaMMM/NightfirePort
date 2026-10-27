# level-14-east-walkway — the SkyBridge Q-Claw crossing resolved and walked green; the route-lid replay reconciliation lands on the test's side; the east stairwell foot and the mouths' lane pinned

Branch `s/level-14-east-walkway` = `origin/s/level-14-lid-and-climb` (9e126532) + `origin/master`
(4c1f5ffb) + the work below. HEAD at report time: 4eacafb6.

## 1. The reconciliation: the panels test is RIGHT, the lid-and-climb §1 claim was wrong [M]

`SP_LoadScript` 0x1dbc08:46-54's mode-0x40 preload is `Script_Play` + ONE `Script_Update` tick +
`Script_Stop` (state 1). `Script_Update` 0x1da998 advances the streams only while
SCRIPTINFO+0xab2 bit 0 (playing) is set (its entry gate is `(+0xab2 ^ 1) & 1 == 0`), and
`Script_Stop` 0x1db668 clears exactly that bit — so after the preload the stream cursors are
FROZEN at t≈1, not resting at their End ops. The state-1 trigger arm ([M] `SP_Update` 0x1dbda8
case 1) calls only `Script_Play` (no rewind: `Script_Set2Start` 0x1daf00 is reached only through
`SP_LoadScript`'s already-loaded arm), so the use-press replay RESUMES from t≈1 and runs the
script to its end — the key-61 ladder art 0x02000751 DOES spawn, and only then does the stay-1 SP
kill itself (case 2's +0x35 arm → state 5 → obj+0xfe |= 1). The port's implementation
(crates/nf-game/src/nis.rs: `load`'s mode ∉ {1,8} arm = start+update+stop+state 1; `update`'s
state-1 arm = `s.start(looping)` with no rewind) matches the decompile, and
`panels_07000014_live::the_route_lid_use_spawns_the_ladder_art_beneath_the_opening` (master, green
here) pins it end to end. No test or port change needed; the wrong claim is corrected in
docs/reports/level-14-lid-and-climb.md ("Correction" section). The lids still never open (the slab
0x02000750's keys are identity [D probe14_spdump]) — every other §1 conclusion stands.

## 2. The west→east crossing is the SkyBridge Q-Claw [D placement + M mechanics]

The start nook's east wall (x 41.53) and the east walkway's west wall (x 44.58) run the full
z 63..67 with no body-height gap [D probe14_cross's horizontal rays at y 2.8/3.3 — the [L] block
the lid-and-climb session measured]. The crossing is the placed grapple point
"GRAPPLEPOINT_SkyBridge" (45.29, 5.10, 62.94) [D placement; the port resolves
[45.2888, 5.0961, 62.9362]] — the name IS the mechanic:

- A bolt-legal aim exists from the standing stance (40.8, floor 2.40, z 65.0-65.5) at pitch
  ≈ 0.20 rad; the direct ray hits the point's mount, the lower arc clears it [D probe14_aim's
  scan over stances and arcs].
- The Q-Claw bolt latches only a class-0x51 object [M Bullet_Delete 0x12e660:50-55 →
  Player_SetGrapplePoint 0x195da0; the bolt's radius 1.0 grows the window, M 0x12c5a0:92].
- The reel pulls 0.05→1.5/tick and HANGS at distance < 2.0 [M Player_SSGrapple 0x192860:79-121,
  :91-94]; the hang (≈(44.05, 4.53, 63.55), d 1.48) stands over the OPEN 3 m slot gap between the
  walkways [D probe14_room], and the release [M 0x192860:56-66] drops Bond into it onto the
  trench floor — the nav lane node 5 (43.05, 0.4, 62.86) [D].
- The port's grapple hang integrates no swing, so Bond lands on the slot's 2.26 rim graze first;
  an east press + hop breaks it and completes the drop [D probe14_room's z 63.8-64.2 band].

## 3. The route walked green (real input, no teleports)

`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot` — UN-IGNORED, rewritten,
GREEN (7381 ticks, 0 stuck, 0 falls):

1. Start → the crossing stance → Q-Claw latch → reel → hang → release → the trench floor
   (landed (46.00, 0.015, 63.22)).
2. The trench east under the east walkway to the corridor (reached (48.17, −0.05, 62.90)) — the
   nav lane node 5 → 4 [D]; floor −0.60 [D this session's grids].
3. The corridor south to the stairwell lane, one jump WEST over the −2.40 pit band
   (x 46.5..47.5 [D probe14_corridor]) onto the −0.91 landing — the EAST STAIRWELL FOOT
   (reached (45.14, −0.10, 53.25); the steps z 58 (2.40) → 57.5 (2.30) → 56-57 (1.05) →
   54.5-55.5 (0.60) → 51.5-54 (−0.91) [D probe14_stairs]).
4. Back east over the pit band, the corridor south to the mouths' lane (reached
   (48.34, −0.05, 44.43) [D nav node 3 → (49.1, 0.4, 42.8)]).

Four assertion-free probes committed as the [D] evidence tools: probe14_cross (the walls + the
fine floor grid), probe14_aim (the bolt-legal aim scan), probe14_south (the south vent lanes +
the TWO shafts — the east shaft throat x 44.75..45.75, z 49..50 — and the stairwell roofs),
probe14_corridor (the y-2.0 floor grid of the corridor + the pit band + the eastward ray scan).

## What is left

1. The stairwell's upper flights walked: the 1.05 flight's continuous west edge (x 44.5..44.75;
   the 45..45.75 middle is a deep throat [D probe14_corridor's y-2.0 grid]) then 2.30 → the
   walkway head at z 58 — my climb attempts wedged in the x 44.4 slot; needs one probe pass to
   pick the step targets.
2. `walk_the_infiltration_to_the_delta_exit` still walks the OLD leg-1a body (the sealed west
   descent) and stays ignored with an updated note: swap its legs 1a-1d onto the pinned east
   route + the mezzanine entry from INSIDE the east duct — the east shaft (throat
   x 44.75..45.75, z 49..50 [D probe14_south]) up to the east vent lane, then south out of the
   mouth over its flush lid (the mouths' Q-Claw climb from below is refuted [M — the lids plus
   the mezzanine strip z 41..43.6 block every corridor ray]). Legs 2-6 are untouched and were
   green on the old route.
3. The full four-crate suite on the final tree (see below — it did not fit this window).

## What ran

- `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -- --test-threads=1`:
  nf-game all green (122 passed across its binaries); nf-formats green except the 3 documented
  `disc_golden` fails (the slim ISO's missing cross-check dumps — the allowed set); nf-audio all
  green.
- The touched nf-view binaries: `panels_07000014_live` 2/2 (the route-lid replay pin — the
  reconciliation), `walkreal_07000014_live` 2 passed / 1 ignored (the un-ignored east route +
  the grill pin).
- The full four-crate suite (`tools/test/suite.py`) was started three times (background starts
  are killed by the sandbox between tool calls — the same failure the trench and lid-and-climb
  sessions logged) and did not fit the session's remaining window; the next session should run
  it foreground-sliced on this tree (HEAD 4eacafb6).

## Live checks for the main agent

Fresh boot 07000014:

1. gdbwatch SCRIPTINFO+0xab2 for the route lid SP #849 while pressing E inside the duct
   (51.58, 3.50, 45.60): expect bit 0 set by the preload, cleared by Script_Stop, set again on
   the use, and Script_CreateEntity hitting the key-61 op ~60 ticks later (the ladder art) —
   this pins the §1 correction end to end on the real game.
2. Stand the game's Bond at (40.8, 2.4, 65.5), fire the Q-Claw at the SkyBridge point, and hold
   east during the hang before releasing: does the REAL game swing (the wish-velocity commit
   [M 0x192860:56-66]) and land ON the east walkway (x ≥ 45)? The port's hang integrates no
   swing and lands in the slot — if the real game lands on the walkway, the stairwell head leg
   becomes a plain descent and my "foot-first" reading is only the port's shape of the route.
3. At the east stairwell head (45.5, 2.3, 57.5): the game's Bond walking down — which x lane
   does the descent hug (the 1.05 flight's west edge x 44.5..44.75, or a slab-covered middle)?

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
