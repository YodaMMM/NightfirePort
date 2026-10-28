# WORKLOG

## level-14-west-door (2026-10-28, branch s/level-14-west-door)

Task: 07000014 (Countdown Part 1) — walkreal_07000014_live
`walk_the_infiltration_to_the_delta_exit` wedges at the facility's west double
door at (-52.2, 1.2, 40.1): two door records, kind 2, state 0x2, pos01 0.00,
ch_open 0, ch_lock 0x85, lock_flag 0. Find how the game opens it ([D] the
placements writing channel 0x85 and their triggers; [M] the door's lock/unlock
and use paths, Door_Update / the lock channel read) — a switch, an objective
step, a script, or an approach side the port lacks — and port the missing
link. Then walk legs 2..6 of the test on real input to the Delta exit #944 and
un-ignore it (or re-ignore on the next exact blocker with its evidence). Full
suite before push, quote the total line.

Plan:
- Start the fast runner in the background in the first minutes.
- Read the wedged test + door records; census the 07000014 placements for
  channel 0x85 writers and their triggers [D].
- Read Door_Update / lock channel read / use paths in work/decomp [M with asm
  where the C looks incomplete].
- Port the missing opener; write real-input tests; un-ignore legs 2..6.
- Full suite (cargo test crates + fast runner), push, report, finish.

### level-14-west-door progress (2026-10-28 ~09:00 UTC)

- The 0x85 mystery RESOLVED: touch `oDroneTrigger1b` (map 54 placement 399, kind 0xEB) at
  (-60.30, 0.61, 38.37) has params (0,160),(1,4) -> out 0xA0, mask 4 [D]. Mask bit 0x4 is the
  E-USE flag: `Trigger_Activate` 0x1b2318 (types 1/2) requires `mask & 4` then writes
  `switch_channels[out]=1` [M]; `Player_Activate` 0x1a5170 case 0x1D -> Trigger_Activate [M].
  The panel-sequence "mask matches nothing" touch is an E-USE SWITCH, not a proximity touch.
- Chain [D]: fuseboxes 0x87+0x88 -> AND -> 0x8C; E-use -> 0xA0; SEQ_AND plc 393 [0x8C,0xA0]->0x51;
  kill cascade plc 400(0x51->0x52) -> 787 GENERATORS(0x52->0x4e) -> 401(0x4e->0x53) -> ... ->
  plc 403(0x5b->0x85); Door_Slide_11A/B plc 151/152 (p3=133=0x85 lock, group 99, approach 0)
  unlock on the rising edge and auto-open [M Door_Update 0x1363c8:88-107 -> Door_Activate].
- Scripts ruled out [D]: no script stream in 07000014/078/079/07A00014 writes 0x85/0x51/0xA0
  (nf-extract nis census; only 0x92/0x93/0xb7/0x8a/0x47/0x0e/0xb8/0x48/0x10/0x11/0x4a writes).
- Port status: door.rs has the rising-edge auto-open; triggers.rs activate_request has the
  mask&4 E path; props.rs channel_kills (panel-sequence) cascades. Remaining: make the walk
  test E-use the touch after the fusebox cuts and route legs 2..6 without the pre-unlock door
  crossing; run and un-ignore.

### level-14-west-door final (2026-10-28 09:25 UTC)

- MISSING LINK FOUND AND PINNED: the west double door's 0x85 comes from the kill cascade whose
  root is touch `oDroneTrigger1b` (map 54 placement 399, out 0xA0, mask 0x4) — mask bit 0x4 is
  the E-USE flag [M Trigger_Activate 0x1b2318:23-33; Player_Activate 0x1a5170 case 0x1D], not a
  contact class [M Trigger_Hit 0x1b1fc8]. It is a switch the player presses E on; SEQ_AND 393
  [0x8C, 0xA0] → 0x51 arms the cascade. The 1-tick pulse suffices (Trigger_Calc 0x1b1900 clears
  type-2 touch outs per frame; the newest-first control walk reads the pulse one frame late
  [M 0x134698 + 0x1336f8]).
- No product code changed — the port already had every piece. New green test
  `the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door` (walkreal_07000014_live):
  E through the real use path → 0xA0 → 0x51 pulse → cascade → 0x85 → the door unlocked, lock
  field cleared, pos01 > 0.5. Binary total: 4 passed / 0 failed / 1 ignored.
- The delta-exit walk stays #[ignore]: leg 2's nav route crosses the locked door plane (the
  next blocker is the route, not the mechanics) — the report carries the waypoint fix plan.
