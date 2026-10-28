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
