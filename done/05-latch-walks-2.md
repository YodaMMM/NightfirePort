# Guards turn off Bond on contact: the last three reds

Branch `s/latch-batch` is current master plus the drone contact-turn rule and its first walk repairs (read docs/reports/latch-walks-on-master.md on it: sections "Failure 2 sisters", "Failure 4", "Failure 5" and "What's left"). Its full suite fails three tests that pass on master:
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent (and the Agent sister if red): guard #33 now parks at contact distance and fires point-blank on the 23.0 strip.
- walkreal_07000008_live::walk_the_whole_mission_on_real_input: not yet diagnosed.
- fix_mission_live::the_07000001_exit_works_end_to_end: the courtyard shoji ping-pongs 0x8/0x10 and never opens (NDrone2_DoorIsOpen 0x14aaa0). Log the shoji state and every drone activating it per tick on master and on the branch; this one may be a port bug in who re-activates the door.

For each: decide port bug vs lost walk margin with evidence, fix the port (test red on old code) or re-route as a player would. Never lower damage or health, no loosened pins, no new ignores. Branch from origin/s/latch-batch, push to `s/latch-walks-2`, report in docs/reports/latch-walks-2.md. Run each touched binary alone, then walkreal_07000001, 07, 08, 09, 11.

---
claimed-by: sandbox1 at 2026-11-08T23:28Z

---
## Report (done, 2026-11-09T00:27Z)


---
## Report from the worker (done, 2026-11-09T00:27Z)

# latch-walks-2 — report

Branch `s/latch-walks-2` from `origin/s/latch-batch` (4b626c80c). Task: the three reds on that
branch — walkreal_07000011's real-damage Agent walks (guard #33 point-blank on the 23.0 strip),
walkreal_07000008's whole-mission walk (undiagnosed), and fix_mission's 07000001 exit (the
courtyard shoji). For each: port bug vs lost walk margin, with evidence.

## Done — the 07000001 exit red is fixed (fix_mission_live 4/4)

**The shoji was never the problem.** Per-tick door probes (temporary env-gated dumps in
`Doors::activate`/`Door_Update`, `NF_DOORPROBE=1`, reverted before the commit) show the
courtyard shoji (door #1, kind 2 slide, (7.9, 5.3, 11.1)) healthy on the branch: Bond's E
opens it once (`ACT f3406 st0x2 fptrue`), it reaches OPEN and sits there ~360 ticks
(`AUTOCLOSE f3797 st0x4`), then closes. **No 0x8↔0x10 ping-pong exists on this branch** —
the old diagnosis (an activation every tick re-closing it) does not reproduce; the port's
activation paths already carry the game's gates (the drone drain `NDrone2_OpenDoor 0x14aa38`
gates on `state & 0x12` [M 0x14aa38], and `Door_Activate`'s state-4/8 arm [M 0x135be0] is
only reachable from an ungated activator, which neither Bond's gated press nor the drone
drain is).

**The real failure is the escort's death.** A second probe (`NF_MISSIONPROBE=1`, in the test,
removed with the fix) logged Mayhew per 20 ticks through the exit loop:

- Loop start: Mayhew at (−10.08, 6.34, −6.62), state 0x1f LeadHide, **health 10.0** — the
  ambush fights' stray rounds had already cost him 90 (the game lands rounds on any drone of
  the other side they meet [M `NDrone2_DealWithObjHit` 0x13fff0:181-232]).
- t20: health **−65.0**, state 0x44 DEATH_ANIM. The shooter: drone **#9**, a fourth hall
  guard the named-three loop never targets (ch99 needs only the 0x11/0x12/0x13 deaths), alive
  at 12 hp, 5.2–5.8 u from the escort, cycling COMBAT_NO_SIGHT 0x5c → AIM_STAND_FIRE 0x68 →
  STEP_AIM_RIGHT 0x74.
- t200: health 0, state 0x47 DEAD. Bond's route follower parks him on the corpse; the ride
  never starts; the exit loop runs out with `door_opened` false (the shoji's OPEN stretch
  happened back in the ambush phase, outside the watched loop).

On master the same exchange killed #9; with the game's contact-turn rule the fight geometry
moved and he survived it at 12 hp. **Classification: lost walk margin, not a port bug** — a
live guard 5 u off the escort path shooting the ally is the game's own rule; the walk must
clear the room as a player would.

**Fix (test-only, `crates/nf-view/tests/fix_mission_live.rs`):** after each named guard dies
— while the ally-wait hold still parks Mayhew at node 33 [M `NDrone2_ReachedDestNode`
0x153358:85-94] — the walk sweeps every living started team-1 drone within 30 of the held
escort with the same real path the named loop uses (`hall_restage` → `walk_nav_to` →
`shoot`, `hall_ammo_ladder` between attempts; excluding the named three; no damage/health
writes, no teleports, no loosened pins). A post-release sweep was tried first and is not the
answer: he rides into the sweep's crossfire and died during it (measured).

**Result:** `fix_mission_live` **4 passed / 0 failed** (was 3/1), re-verified after probe
removal (146.99s run).

## Classified, not fixed — walkreal_07000011 (00 Agent only)

Re-ran the binary: **the Agent sister is GREEN now** (`walk_spawn_to_the_lower_floor_load_
point_real_damage_agent ... ok`); the red is `..._00agent` only (3 passed / 2 failed lines,
the failure list names 00agent twice). Bond still dies on the 23.0 strip (tick 6274 at
(15.78, 24.09, −2.42), 21 hits for 100.0; #33's ledger 56.0 in 8 hits at d 2.2 — three 7.0
point-blank rounds at frames 5967/5974). The predecessor's classification stands (margin
loss: the game's latch rule parks #33 at contact distance where master's slide carried him
past; Hard accuracy 1.0 out-damages the return fire — the Agent run at 0.7 survives the same
geometry, which is the cleanest evidence the port behaviour is shared and only the difficulty
margin differs). The player-shaped fix is still the flight-A mouth stand (measure the strip
guards' eye-ray exposure from the mouth, then fight the pair from cover) — not landed this
session; it needs the exposure measurement and a re-run of the difficulty, and the remaining
time went to the shoji fix above.

## Diagnosed to the door, not fixed — walkreal_07000008 whole mission

Re-ran the binary: only `walk_the_whole_mission_on_real_input` (difficulty 1) is red; the
`real_damage` and `real_damage_agent` sisters are green and the 00agent sister is ignored
with its documented Hind-fight state. The d1 walk dies at **the west ledge at the cabin**:
Bond E-presses the door at (154.5, 42.0, −121.3) for the 3000-tick cap and the door never
answers — STUCK dump: `kind=0 state=0x2 pos01=0.50 ch_open=0 ch_lock=0 lock_flag=0
approach=0`, and drones **#7/#8** (team 1, state 0x63 GO_TO_GOAL, hp 10) parked at
(154.0, 43.0, −121.0), in the doorway — the contact-turn rule's doing again (on master they
slid through). The walker's `use_door` is already patient (presses one edge per 6 ticks only
while `state & 0xc == 0`, breaks on 0x8 [M `NDrone2_OpenDoor` 0x14aa38's `& 0x12` gate is
the same set]), so the door is REJECTING or bouncing activations with a parked swing-path
occupant — `Door_Activate`'s kind-0 arms with an activator in the swing path set the 2-tick
busy marker only [M 0x135be0; `Doors_Calc` 0x135fe0 clears it], and a mid-swing re-close to
0.5 would explain `pos01 0.50`. The `NF_DOORPROBE` probe is built for exactly this; the
283-second whole-mission run with it did not fit the session. Next session: run d1 with the
probe, read who activates door (154.5, 42.0, −121.3) and with what state transitions, then
decide port-vs-route with the same evidence chain as the shoji.

## Verification (every binary run whole, this session, this branch)

| binary | result |
|---|---|
| fix_mission_live | 4 passed / 0 failed (was 3/1) — after the fix, twice (probe present and removed) |
| walkreal_07000001_live | 7 passed / 1 ignored / 0 failed |
| walkreal_07000007_live | 5 passed / 1 ignored / 0 failed |
| walkreal_07000009_live | 9 passed / 0 failed |
| walkreal_07000011_live | 3 passed / 2 failed (00agent named twice — agent sister green, plain + Operative real-damage green) |
| walkreal_07000008_live | 4 passed / 1 failed / 1 ignored (only the d1 whole-mission red, above) |

The full suite did not fit the session (setup consumed ~40 min: sandbox reset, py7zr install,
ALSA headers, ISO download; the whole-mission walk alone is 283 s and was run twice). No
shared code changed: the diff is `crates/nf-view/tests/fix_mission_live.rs` only (door.rs and
walker probes reverted; `git status` clean of scratch). The touched-binary rule is satisfied
by the fix_mission runs; the walkreal set above covers the required list (01/07/08/09/11).

## What's left (next session)

1. walkreal_07000008 d1: the `NF_DOORPROBE` run on the cabin door (above), then port-vs-route.
2. walkreal_07000011 00agent: the flight-A mouth stand (measure exposure first), re-run.
3. The full `cargo test --offline --release -j2` suite before the next push.

## Live checks for the main agent

1. **07000001 escort survival** — during a real playthrough of the ambush + ride, read
   Mayhew's health (Drone+health) and hall guard #9's state (+0x? state byte, expect the
   0x5c/0x68/0x74 cycle the port shows): does the game's own #9 survive the ambush and shoot
   the escort, or does he die in the exchange (port vs game fight outcome)? Access: read,
   per frame around the ch99 release.
2. **07000008 west-ledge cabin door** — with guards #7/#8 parked in its doorway, read the
   door's `obj+0xF4` (state) and `obj+0x6C/flags` per frame while a player presses E:
   expected if the port is faithful, the door opens (state → 8 → 4); if the game's door also
   stays 0x2 with pos01 0.50, the port is faithful and the walk needs the re-route.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for
the session (no usage readout is shown in the IM gateway or the tool output).
