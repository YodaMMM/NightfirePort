# level-12-alarm-guard-walls — report

Branch `s/level-12-alarm-guard-walls`, cut from origin/master c0450ef6. Sandbox was reset at
start: the repo, `work/`, the ISO, rust and the ALSA prefix were all re-set-up in foreground
(work-slim.7z + the ISO via py7zr; rust 1.99.1; libasound2-dev extracted to
`/home/z/alsa-prefix`).

## What's done

**The wedge spot is named to geometry (committed probe `crates/nf-view/tests/walkreal_probe12f.rs`,
log-only, `#[ignore]`d).** (22.97, 1.06, 36.21) is INSIDE the real cafeteria room of 07000012:

- static collision [D scans]: floor y 0.0 under the spot, ceiling hit at y ≈ 3.13; a static
  wall at x ≈ 18.8–19.8 on the z 36.2 line (the room's west wall); no floor north of z ≈ 36.6
  (the void beyond the room);
- nav data [D]: the room is covered by the Default Navigation path — nodes path#8[9]
  (22.06, 0.45, 33.34), [180] (25.33, 0.52, 35.80), [181] (19.35, 0.52, 34.18) all within 3 u;
- the BLINK boundary walls #0–#5 bound the room [D]: #1 (37.23, 0.42, 36.66)→(22.47, 0.42,
  36.64) h 2.53, #2 →(20.00, 36.14), #3 →(17.99, 34.82), #4 the z ≈ 32.0 south edge, #5 the
  corridor-mouth diagonal. **The wedge spot sits 0.43 u south of the wall#1/#2 corner — and
  BLINK walls are AI boundaries, NOT drawn geometry. "Nothing is drawn there" is exactly what
  a BLINK corner looks like.** The only opening between the room and the offices is the
  corridor mouth (x ≈ 17–19, z ≈ 31–33.5).

**The game's alarm + movement path is read and cited:**

- the alarm broadcast [M Drone_InitComms 0x139520:255-259]: while `switch_channels[0x96]`,
  every 30 frames `Drone_Message(0, 0x1e, 0, 0)`; [M Drone_Message 0x13a678:21-23] with obj 0
  → `Drone_SM_BroadcastMsg` → [M Drone_SM_RouteMsg 0x172128:106-142] delivers to every drone
  whose class word `obj+0x10c` is NOT in {1, 3, 0xf, 0x47..0x48} — **no enabled gate**;
- [M NDrone2_DSTATE_WaitSwitch 0x15b960] handles msgs 1 (enter → Enable(0)), 3 (update: the
  wait channel on → Enable(1) + the start state) and 0xe (ENABLE → Enable(1) + the start
  state); **msg 0x1e (ALARM) hits the default `return 0` — a WAIT_SWITCH guard ignores the
  alarm**. The port's alarm-rebroadcast filter (`present && enabled`, drone.rs:18101) and its
  WAIT_SWITCH arms (drone.rs:7328-7330) are therefore behaviour-faithful here;
- [M NDrone2_Collision 0x140e90:169-174]: when drone+0x4f8 has the sign bit, the GAME ITSELF
  boundary-pushes every bounded guard per tick via `AINetwork_GetBoundsPushVectorForSphere
  (r 0.4, pos, room, …)`. The port mirrors this (nav.rs `push_out` [M 0x157bd0], applied at
  drone.rs:19789-19796 behind the `bounded` flag).

**Three headless repros were built and run (the committed probe, real input, fixed 60 Hz):**
alarm-charge with Bond walking the walkreal route south; Bond walked north THROUGH the corridor
touch trigger plc 30 (its out ch 0x5 fires [D], enabling the west-wing pair placements 21/22
[D params]) into the cafeteria, then standing at (21.75, 34.42); and a whole-roster stress
where every WAIT_SWITCH guard is enabled through the game's own msg 0x0e and then alarmed.
Result in every run: the corridor pair (#8 plc 12, #9 plc 13) and the lobby pair (#28/#29 plc
691/692) advance only to the corridor mouth (z 24–31) and park in stand/crouch/cover-fire
(one emits the cover goal (13.06, 30.65)); the west-wing pair patrols its own wing; **no guard
ever enters the cafeteria, and no guard wedges at (22.97, 36.21) in any run.**

**The reading (current best):** the reported press needs guards INSIDE the cafeteria — which
the port never produces in these repros. The press position itself (0.43 u off the BLINK
corner, vs the port's push_out r 0.4) strongly suggests that IF the game's guards advance into
the room and are then boundary-pushed there, the port's `push_out` would hold them at the same
spot — i.e. the port may be faithful at the corner and divergent EARLIER (the advance law).
The open question is the game's combat-advance law (COMBAT_OUT_OF_RANGE 0x5a approach /
COMBAT_NO_SIGHT 0x5c search toward the last-seen position) vs the port's fire-state park at
8–14 u with no line of sight.

## What's left

1. The live checks below — they settle which side deviates; the fix (either the port's
   combat-advance pick or a slide-along-wall refinement in `push_out`) follows from them.
2. The four-crate suite on the final tree: `tools/test/suite.py` was started in the background
   per the task line, but this sandbox kills background jobs between tool calls (suite.out
   stayed empty twice; the build cache persisted). The named binaries below ran in foreground.

## Live checks for the main agent

1. **The corridor guards' advance law** (fresh boot, alarm raised, Bond walking the route
   south): gdbwatch placements 12/13's `obj+0x5A2` (return state), `obj+0x554` (route),
   `obj+0x670` (move target) and `obj+0x30` (pos) for ~10 s after they first fire. Expected:
   whether the game's guard ADVANCES toward Bond (the 0x5a approach) or holds a stand/crouch
   fire park at 8–14 u with no line of sight — the port parks; if the game advances, the
   port's combat-move pick is the fix site.
2. **Do game guards enter the cafeteria?** Same play, Bond walked north through the corridor
   (crossing touch plc 30) into the room, standing at (21.8, 34.4): do game guards reach
   z > 32, and where do they stop? If they press near (22.97, 36.21): gdbwatch
   `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0 (called from 0x140e90:170) for the wedged
   guard, and read `obj+0x670` while pressed — if the game's move target is SOUTH (toward
   Bond) while the guard presses the north corner, the game SLIDES along the wall and the
   port's XZ pin (`push_out`, nav.rs:142) is the divergence; if the game's target is also
   north/into the wall, the port matches the game.
3. **Render check** (port, no code change): `--bond 21.8,1.1,34.4` facing north — confirm
   nothing is drawn at (22.97, 1.06, 36.21) (the BLINK corner), matching the driver's frame
   1391 observation.

## What I ran (this branch, release, NF_ISO the slim on-foot image)

- `walkreal_07000012_live`: 2 passed / 0 failed (plain + real-damage walks).
- `interactables_07000012_live`: 12 passed / 0 failed.
- `nf-game --lib`: 655 passed / 0 failed / 5 ignored.
- The probe binary `walkreal_probe12f` builds clean and ran as the three log-only repros above.
- The background `tools/test/suite.py` line was launched twice per the task instruction; the
  sandbox kills background processes between tool calls, so it produced no total line. The
  full four-crate run did not fit the remaining window after the from-scratch setup (clone +
  work + ISO + rust + ALSA ≈ 40 min of the 2-hour budget).

## Usage

not visible to me
