# level-05-every-objective-4 — report

Task: land the 05 walk extension. `origin/s/level-05-every-objective-3`'s new
`walkreal_07000005_live::the_door_band_exit_chains_to_07000006_climb_parked` failed on the
master merge ("the terrace placement stands on the y-49 band … got [21.805223, 50.035336,
−68.09826]", walkreal_07000005_live.rs:1361); find what Bond stands on at y 50.04 at
(21.8, −68.1), fix the test or the port with evidence, then continue the remaining legs
(the east stairs, door #112's E → 07000006). Branch `s/level-05-every-objective-4`
(base origin/s/level-05-every-objective-3 9dc521bc, merged origin/master f82fa6cc →
a27e53f0).

## What's done

**1. The terrace placement failure: the TEST was wrong, with the full evidence chain.**
The -3 draft placed Bond at (22.0, 49.2, −68.0) and asserted the centre `|y − 49.0| < 1.0`.
Three independent facts say a standing centre can never be at the floor band:

- [D] The disc's terrace floor at (21.805, −68.098) is **49.001** (instance 20, map 50,
  object 38, mat 12) — the y-49 band IS under the spot (`probe_terrace05`, committed; its
  MAP A also maps the whole terrace: x 14..35.5 at 49.0, z −65..−69.5, the y-50/51 rim at
  z −70..−71, the staircase 37.2→43.3 at z −71.5/−72.5).
- [L] The real game parks Bond ~1.086 OVER his floor: `ee_start_05.bin` boot state has
  obj+0x30 = 24.9633 at (104.954, 108.825) while the disc floor under that column is 23.877
  (the same read the repo already records as the 05-truck trace "standing live 1.086, cc
  1.0347", player.rs ROOT_HEIGHT doc).
- [M] The port's upright capsule bottom = pos.y − cc (A = +0.275, B = +0.55 − cc, r 0.55 —
  Player_Collision 0x194df0), so standing ON the floor puts the centre at floor + cc =
  49.0006 + 1.0347 = **50.0353** — exactly the -3 run's "got". The 0.051 delta to the game's
  live 1.086 is the documented standing-rest offset (player.rs:18).

The -3 placement also BURIED the capsule (B = 49.2 − 1.0347 = 48.17, 0.83 under the floor),
so the push lifted AND shoved him west to x 21.805. Fixed: place at the rest height
(22.0, 50.1, −68.0) — no interpenetration, zero drift, settles 50.089 (the feet-band rest,
[M] Player_FeetOnPoint 0x194188:106 `hit − 0.1 <= cc`) — and assert `grounded && y in
(49.9, 50.4)` with the evidence in the comment.

**2. The door #112 chain now fires end-to-end: E → fallback ch 0x1E → level exit
07000006.** The -3 draft's threshold waypoint (35.4, 49.0, −69.0) stopped Bond at x 34.88;
his use sphere reached only x 36.88 — 0.24 short of the exit trigger's face. The chain:

- [D] Placement #112 kind 0xE8 "Locked_Door_Top_I" @(37.25, 49.00, −69.00): p0=0x7000006,
  p1=0, p2=0, p3=0x1e, p4=0 (probe_placedump05).
- [M] Trigger_Create 0x1b1720:67-77 maps p0→ext+0x20 (the level id), p1→+0x26 (key gate),
  p2→+0x28 (the auto-fire gate), p3→+0x2a (the FALLBACK channel), p4→+0x2c (success).
  So #112 is unlocked (key gate 0), never auto-fires (gate 0), and its E press falls to the
  +0x2a fallback write: switch_channels[0x1E] = 1 [M Trigger_Activate 0x1b2318:91-107], then
  ResetMap_LevelToLoad(0x7000006) (:108-117).
- [M] The reach: Player_Activate 0x1a5170:100-108 — a radius-1.0 sphere at head + 1×view
  dir vs the cel's object list; the class-0x1d arm calls Trigger_Activate (:134-137). The
  port's AABB stand-in over the 0xE8's glist spans x 37.12..37.38, y 49.0..51.25,
  z −69.0..−67.75.
- The fix: the walk target is the arch itself (37.0, 50.1, −68.4). The doorway is open
  floor — no solid door object blocks the crossing ("Locked_Door_Top_I" names the level-exit
  TRIGGER [D kind 0xE8]). First E press: ch 0x1E on, level_exit 07000006 (the test's own
  eprintln logs it).

**3. The whole file is green: walkreal_07000005_live 10 passed / 0 failed**
(`--test-threads=1`, slim ISO, NF_SILENT=1; 99.31 s). That includes the big walk
`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` (the corridor → spine → SW
loop → #269 → grapple → landing legs all green) with the east-stairs legs still PARKING at
the ch 0x42 script window as a documented boundary (below).

**4. The ch 0x42 script window is decoded one level deeper (the parked east legs).**
New evidence for the -3 report's open port gap, from NF_SCRIPTLOG=1 on the walk run [L] plus
the disc data [D]:

- The chain: SWITCH #269 (out 0x42) → the grapple landing (tick 8723) → script player #284
  (kind 0xD9, p0=0x06000952 [D placement]) starts NIS 0x06000952 (camera script, length 100
  frames, 11 streams [D level.nis]).
- Its stream 6 raises **event 4 = Player_Disable** [M 0x1d94c0:118-121] at script frame
  8757 — the sim's `disabled` flips on; stream 8 raises **event 0x0D → script channel 0x62
  = 1** at clock 93; stream 8's end op sits at t=100 and the script reaches `play=done`
  (~tick 8850). Then nothing — no free, no enable, `disabled` stuck for 3600+ watched ticks.
- The script contains **no event 6** (the player-return event) anywhere — the only events in
  all 11 streams are ev4 (disable), ev5, ev0xD[98,1] (probe_script05, committed). So even a
  Script_Free of 06000952 would push no PlayerEnable [M Script_Free 0x1da880:44-49 is gated
  on the event-6 flag 0x100].
- [M] The game's own SP_Update case 2 (0x1dbda8:143-154) ALSO ends a stay=0 player in
  state 3 with no Script_Free — the port matches the decompile here. And no 0xD9 placement
  carries 0x62 in any param (mask p5 / gate p9 — the param sweep), and the helicopter's own
  NISs 06000046/06000047 (on #241/#242, kind 0x2F, p6=0x42 [D]) contain no event 6 either.
- Conclusion: the real game's hand-back after the helicopter-start cut-scene rides a
  mechanism outside every script this chain decodes (nothing in the port's reach sets
  flags 0x100 or calls Player_Enable). This is exactly what the live check below must
  answer; a port-side guess would invent the game's behaviour.

## Live checks for the main agent

1. **The ch 0x42 hand-back** (the -3 check, now sharpened): fresh boot 07000005, poke Bond
   to the battlement's east end (−16.4, 38.2, −72.4), grapple east to pulley #281 and land
   (the port lands at (1.10, 37.62, −72.58)). Watch: does the real game disable input at the
   landing (the port's SP #284 → 06000952 ev4 at its frame 8757)? How long? Where is Bond
   when control returns, and WHAT mechanism re-enables (a Player_Enable from which script —
   the port finds no event 6 anywhere in 06000952/06000046/06000047 and no consumer of
   script ch 0x62)?
2. **The stairs→terrace climb** (the -3 check, unchanged): fresh boot, poke Bond to the
   staircase top (35.0, 43.4, −71.4), walk north — how does the game get onto the y-49
   terrace (no walkable link in the disc's collision at 0.25 resolution [D probe_corridor05
   MAP 8-12 / probe_terrace05 MAP A])? The door-chain test still pins the EXIT from a
   probe-grade terrace placement, clearly marked in its doc.
3. Confirm the terrace walk east through the arch at (37, 49, −69) and the E press on the
   real game (the port's chain: ch 0x1E → exit 07000006, pinned [M/D] above).

## How it was verified

- `probe_terrace05` (committed): the disc's fine floor map over the terrace + full column
  dumps at the settle point, the boot column, and the door columns — the [D] floor evidence.
- `probe_terrace_port05` (committed): the port-side queries — the scoped/flat floor rays at
  the settle column (both 49.0006, normal +y), the trigger dump (#16 kind 10 with its glist
  AABB), the walk-east flow and the E press that fires the exit.
- `probe_script05` (committed): the 0xD9/0x31 placement sweep, the 06000952 stream/event
  decode, the 06000046/47 event decode.
- The test: `cargo test --offline --release -p nf-view --test walkreal_07000005_live --
  --test-threads=1` (NF_ISO set, NF_SILENT=1) — 10 passed / 0 failed, quoted above.

## What's left

- The east-stairs legs (x −4..13 lane + the covered staircase x 14→35.5) are written into
  the big walk and stay green-parked at the ch 0x42 window; they go green when the live
  check answers the hand-back mechanism and the port models it.
- The stairs→terrace climb (live check 2) — the door test's placement stays probe-grade
  until then.

## Suite

The fast runner (`tools/test/suite.py`, 912 test binaries, build ok in 1248 s) was started
in the background at 04:00 UTC and was still running at wrap-up (05:40 UTC window). Its
failures so far are exactly the slim-ISO artifacts: the 3 `disc_golden` tests and
`identity_disc` (the slim image has 518,435 sectors; the identity assert pins the full
disc's 1,191,072 — docs/build-identity.md; the same failure in the previous session's
suite.log). No other FAILED/TIMEOUT lines. **The touched binary total: walkreal_07000005_live
10 passed; 0 failed** (no full suite needed per the task; the run's remaining totals land in
the WORKLOG entry if they arrive before close).

## Usage

not visible to me
