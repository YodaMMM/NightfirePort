# batch3-hard-walks: the batch-3 hard walks re-plan

Branch `s/batch3-hard-walks`, base origin/master c229c7780 (stream batch 3). Sandbox was reset at
05:30 UTC; the full setup ran fresh (clone, work-slim + ISO 369823203 bytes extracted with py7zr,
rustup 1.100.0, the ALSA user-prefix at /home/z/alsa-prefix wired through PKG_CONFIG_PATH +
RUSTFLAGS, cargo fetch, the nf-queue worktree). Every run below is foreground on this box
(background jobs die between tool calls, the known sandbox constraint).

## Done

### 1. walkreal_07000001 `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent` — the hall fight is SOLVED on real input; the walk's end moved ~37000 ticks later
Reproduced the batch-3 ledger death first: t16043 at (7.76,6.00,14.59), shotgun guard #6 (skin
0x5000036, weapon 0x1c) at d 4.8, after a lane fight that landed ZERO (guard still hp 12 at
t15511 with Bond at 2.2) — the fatal press then walked Bond into #6's volley.
Two new fight-plan levers, both d3-gated inside their existing blocks (the d1/d2 sisters are
byte-identical — the whole binary passed with them):
- The d3 lane-stand candidates gained the EAST cells (x 8, z 6.8-10.6) — the walk's own hall-3
  instrumentation grid on this tree lists CLEAR crouched flight lanes to BOTH shotgunners from
  there [L the grid print in the run log]. The WEST cells (4/4.6) were tried first and measured
  FAKE-CLEAR (the probe cell sits inside the solid: the stand walk wedged at the pocket's east
  wall x 3.4, d 1.2 short, at the 3000-tick cap, twice) and are not candidates.
- The per-guard last-guard press is gated on the target being exposed AT the press spot: the #6
  press printed `exposed false` and its shot nav walked Bond from (6.60,6.00,11.91) to
  (7.8,6.0,14.6) — INTO the volley (the t16043 mechanism); the #7 press printed `exposed true`
  and killed him free. A not-exposed target keeps the lane-fight/escort machinery.
Measured [L run5 + the whole-binary run]: both shotgunners die at the east stand (g7h0 g6h0 on
advance leg 4), Bond exits the hall at 57.4 FULL (the baseline exited at 2.2 and died), the hall
captains 5/14/15 die free, and the walk reaches t52740 at 57.4 health + 24.8 armour. The walk's
END is now the CONTACT's own death: the mission-fail restart posts at t52776 with the contact a
corpse at (-30.5,6.3,13.9), 1.4 u from the live pocket SMG #12 (-31.31,11.51) — the pocket lane
fights left #12/#17 alive ("killed false") and the contact cowered in #12's line through the long
guard-2 exposed-false stalls (15 ambush rounds of 540×3 holds). The test stays `#[ignore]`d with
this ledger and the next lever (kill the pocket pair before the contact parks beside it, or
shepherd the contact's park spot out of #12's line).
Verification: `walkreal_07000001_live` run WHOLE — 7 passed / 0 failed / 1 ignored (506 s).

### 6. interactables_07000015 `sister_the_dead_ladder_mesh_reaches_the_point_blank_stander` — fixture REBUILT on the one-sided push, UN-IGNORED, green
First reproduced the ignore's measured claim: on the old pin (85.08, 40.4, -2.4) the one-sided
push settles Bond onto the raised slabs at y 41.11-42.30 where the default arm's capsule rides
ABOVE the rung box — a fresh-load scan grid (x 84.70-85.65 × z -3.5..-0.8 × y0 40.5/41/41.6)
measured reach FALSE across the old scan box. The scan then found the walkway's RAW floor band
north of the plane axis: (84.70/84.90, 40.5, -1.60/-1.20) settle GROUNDED at y 40.50-40.56 and
`capsule_hits_default` reads TRUE there (the rails' lateral span carries the mesh box under the
capsule; the raised-slab stands miss it). The fixture now pins (84.70, 40.5, -1.60) with a
grounded assert, a y-band assert, the ladder-#382 identity assert and the reach assert; the
`#[ignore]` is removed. The teleport only REACHES the spot (the file's standing convention); the
settle, stand and pick are the game's own one-sided push.
Verification: `interactables_07000015_live` run WHOLE — 19 passed / 0 failed / 0 ignored (36 s).

### Fresh ledgers on the untouched walks (measured on this tree, committed to their ignore messages)
- walkreal_07000013 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`: the walk now
  wedges BEFORE its ledger's lobby blocker — the south hall west leg's walk_to hit the 6000-tick
  cap (Bond (6.43,1.12,-19.50), target (-9.75,1.10,-22.25) d 16.4, the segment blocked at t
  0.034, cel Some(242)) at t10080 with 54.5 health after killing #12 (13 hits for 45.5 taken
  before it) [L the fresh run]. The ignore message now names the FIRST lever: this leg's route
  on the joined stream, above the lobby west-mid lever.
- walkreal_07000016 `walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent`:
  re-confirmed the ledger death on this tree — t5005 at (19.11,1.49,-97.63), 35 hits for 100.0,
  3 return-fire fights (the ledger's pocket spot (19.1,1.5,-97.4) at its t3400-8700 band). No
  edit made; the pocket fight needs its own new plan (cover/range/target order).

## What's left (the next round continues from the branch)
- 01: the contact lever (kill the pocket pair #12/#17 before the contact parks in their line; or
  shepherd his park spot out of #12's line; the guard-2 exposed-false stalls are the window).
- 13: the south hall west leg's route (the fresh wedge above).
- 16: the pocket fight plan at (19.1,1.5,-97.4) / (-13,-22,-171).
- 0d `walk_arrival_to_the_c5_door_real_damage_agent` and the three 1b Drake walks: NOT STARTED
  this window (no run, no edit) — the batch-3 ledgers stand as left; each needs its own
  measure-first session (the 0d sister's next levers are already named in her ignore: the
  crossfire-5 lever set on a green Operative base, then the d2 aimed stand on the two d2 snipers).

## Verification notes
- Only two test files changed (`walkreal_07000001_live.rs`, `interactables_07000015_live.rs`,
  plus the 13 ignore-message ledger line); nothing under `crates/*/src/` changed, so no other
  test binary is affected by these edits (the lesson-10 grep: no test calls what changed — the
  changed code is the two files' own internals).
- Both touched binaries ran whole and are reported above with their pass counts. The full
  4-crate suite did not fit in this sandbox window (~100 min total; the two whole-binary runs and
  the scan/build cycles consumed it) — reported honestly per the suite rule.
- No game bytes, tokens, URLs or env scripts committed (`git status --short` checked per commit;
  `env.sh` and the build logs stay untracked).

## Usage
not visible to me (no token, credit or context-window figures are exposed in this session's UI).
