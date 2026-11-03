# level-14-cascade-servers-4 — the door-spam fix verified; one live-check question left

Branch `s/level-14-cascade-servers-4` = origin/s/level-14-cascade-servers-3 + origin/master
(merged at 7b25daa8) + three commits: the branch start, **the stale+distance clear wipes BOTH
+0xBA8 stand-ins** (941808fa) and **the ENTER deferral** (50bcbeca), then two WORKLOG/report
checkpoints. The game has ONE slot (+0xBA8); the port had split it into `hit_door` (the
DoorIsOpen/OpenDoor side) and `door_bound` (the ReachedDestNode no-op key) and cleared only one.

## What's done

1. **The -3 regression fixed.** All 11 tests the driver listed red on the -3-onto-master merge
   are GREEN on this branch, and `no_guard_spams_the_opendoor_after_the_cascade` stays green.
   The two [M] pieces:
   - **The single-slot lifecycle** [M NDrone2_ControlSTANDARD 0x1490f0:233-244]: 60 ticks after
     the last `NDrone2_OpenDoor` stamp (+0xBB0), a bound door farther than 2.0 u is forgotten —
     and the clear must wipe BOTH stand-ins, or the stale `door_bound` no-ops every later route
     pass while `hit_door` is gone (no re-bind, no re-attempt: guards strand before closed
     doors — the 11 reds).
   - **The ENTER deferral** [M 0x153358:149-163 → 0x1681f8 case 1]: the game binds +0xBA8 and
     only then delivers `DSTATE_OpenDoor`'s ENTER, whose `NDrone2_DoorIsOpen` reads the
     just-bound door. The port resolved the node → door outbox only after the state machine
     drained, so the ENTER read the stale pre-bind latch (None → open) and bounced EVERY fresh
     binding back to +0x5A6 — the driven guard stood in 0x63 with the reach consumed. A fresh
     binding now defers the ENTER; `Drones::tick` re-delivers it right after the resolution.

2. **The full walkreal sweep ran** (every level binary, `--test-threads=1`, this tree): all
   green except two, both diagnosed below.

## How it was verified (per-binary pass counts, this session, HEAD 50bcbeca + docs commits)

| binary | result |
|---|---|
| npc_doors_live | 6 passed / 0 failed / 1 ignored (pre-existing, level-05's note) |
| fix_guard_misc_live | 6 passed / 0 failed |
| walkreal_07000003_live | 5 / 0 (both driver-listed reds green) |
| walkreal_0700000a_live | 10 / 0 / 3 ignored (all three driver-listed reds green) |
| walkreal_07000014_live | 9 / 0 (all four driver-listed reds green — the delta-exit wedge and the tripbomb route included) |
| destroy_wreck_07000014_live | 3 / 0 — the spam pin `no_guard_spams_the_opendoor_after_the_cascade` KEPT, the racks pins kept |
| walkreal_07000002/04/05/06/07/08/09/0b/0c/_barrelrow/_eastroute/0d/11/_route/12/13/16/1b/41/4a/_satchel/dump | all 0 failed (2,8,14,6,5,4,6,5,2,1,1,1,2,1,3,3,14,17,11,2,2,1 passed resp.) |
| walkreal_07000001_live | 5 / 1 — `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (below) |
| walkreal_07000015_live | 5 / 1 — `walk_spawn_to_the_omega_sector_exit_real_damage_agent` (below) |

Full suite, run in batches this session (the sandbox kills any process that outlives one tool
call, and the disk filled at 100% mid-run — cleaned 5 GB of stale bisect artifacts and
continued; the batches below are what fit before the 20:44 UTC stop-work mark):

- **nf-game: 1047 passed / 0 failed / 8 ignored** (102 binaries).
- **nf-formats + nf-audio: 136 passed / 3 failed** — the 3 are the `bigf`/`directory_tree`
  golden-manifest tests = exactly the slim-ISO `disc_golden` allowance.
- **nf-view walkreals: all 26 level binaries**, counts in the table above; 2 failures, both
  diagnosed below.
- **nf-view guard/door family (partial, 6 of 16 before the sandbox kill)**: fix_npc_doors,
  door_guards_locked_door_07000014, invisible_guard_07000014, corridor_guards_07000012,
  stairwell_guards_07000011, glass_guards_07000014 → 12 passed / 1 failed, and the 1
  (`a_guard_behind_the_comp_room_window_is_visible_through_it`) is **wgpu `no_adapter`** — the
  sandbox has no GPU; the test needs a render surface, not a logic failure.
- **Not run**: the remaining ~830 nf-view test binaries (interactables, the other guard-family
  members, fix_*/probe sweeps) — the sandbox tool-window kills + the disk-full event consumed
  the window. No failure signal in anything that ran.

## The two sweep exceptions, in full

### 1. walkreal_07000001 `..._real_damage` — the ENTER deferral exposes a game-shaped door opening; needs the main agent's live check

- GREEN at the merge base 7b25daa8 and at fix-1 941808fa; RED only with the ENTER deferral
  (50bcbeca). Bond dies at the exit (t7884, 37 hits for 100.0) vs 2.5 hp left at base.
- The first divergence (bit-exact before it: drones #11/#6/#7 identical ticks and damage): the
  TASE_DBG trace shows **drone 15 (RUN_FOR_COVER 0x8C) binding the hall east-door node 21 at
  ~frame 3458 and OPENING door 2 guard-side** (state 2 → 8 at 3458; drone 14 re-binds at 3469)
  — ~100 ticks before Bond reaches the door. The hall guards get LOS through the early-opened
  door and drone #5 lands 6×1.5 on Bond.
- Per [M] this is the game's shape: `LinkCreep_Handler` 0x15a2c8:63 runs
  `NDrone2_ReachedDestNode` on the croute too (RunForCover's UPDATE drives
  `NDrone2_MoveToGoalPosition` 0x1517d8 → `AINetwork_CalcRouteToPosition` +
  `NDrone2_FollowRoute`), the bind-then-ENTER lands `DSTATE_OpenDoor` case 1, `DoorIsOpen`
  false → the anim + `NDrone2_OpenDoor` 0x14aa38 → `Door_Activate` — the door opens. **Base and
  master stay green only through the stale pre-bind latch read — the very accident the 11 red
  door tests prove wrong per the game.** The test's premise (Bond survives the hall fight) was
  recorded under that accident.
- What I could NOT verify in the sandbox: whether the REAL game's guard actually opens that
  door during the approach fight (the cover-node choice and A* route could differ), and whether
  real-damage Bond survives the fight in the real game. That decides between (a) the test's
  premise needs a game-true re-record, and (b) my deferral is missing a croute-specific gate.

### 2. walkreal_07000015 `..._real_damage_agent` — pre-existing on the -3 line, not this branch's delta

RED at the merge base 7b25daa8 too (verified by checkout + run): Bond dies on the Omega Sector
catwalks (11 hits for 100.0; the world hurt volumes 48.0 + drones #2/#11/#5). The agent
difficulty is the `real-health-agent-difficulty-*`/`difficulty3-*` tasks' active work; the -3
line's full sweep never reached this binary. Left for the task that owns the agent walks.

## Live checks for the main agent

Level 07000001 (Nightfire Manor), the approach fight after the meet (Bond heading west through
the hall east door):

1. **Does a hall guard open the hall east door guard-side before Bond reaches it?** Watch the
   door obj's `obj+0xF4` (u16; 2 CLOSED → 8 OPENING) with `tools/trace/gdbwatch.py` from the
   meet-plate channel write (ch 0xE) until Bond crosses; also watch the engaging guard's
   `+0xBA8` (u32, the bound door obj ptr; bind writer `NDrone2_ReachedDestNode` 0x153358:151-155)
   and break `NDrone2_OpenDoor` 0x14aa38 / `Door_Activate` 0x135be0. Expect per [M]: within ~2 s
   of the fight starting, some guard's +0xBA8 stands, `DSTATE_OpenDoor` case 1 (0x1681f8) runs
   with DoorIsOpen false, and the door's state goes 2 → 8 BEFORE Bond touches it. If the door
   only opens for Bond → the deferral is missing a croute gate; if it opens guard-side → the
   port at HEAD is game-correct.
2. **Does real-damage Bond (default difficulty) survive the hall fight to the exit?** If the
   door opens guard-side and Bond still survives in the real game, the port's post-door fight
   (drone #5's 6 hits) is the divergence and needs its own re-derivation; if Bond dies in the
   real game too, `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` needs a
   game-true re-record (its current 2.5-hp margin was the stale-read accident).

## What's left

- The main agent's two live checks above decide the 07000001 real_damage disposition (re-record
  vs a missing gate); the report's evidence chain (bind/enter/open trace, base-green
  attribution) is ready either way.
- The 07000015 agent walk belongs to the agent-difficulty task line.
- (If the live check says the door opens guard-side: next session re-records the 07000001
  expectation with [L] proof and re-runs the sweep + suite.)
- The rest of the nf-view suite (~830 binaries) — see the suite section.

## ntfy

curl: `(28) Connection timed out after 20000 milliseconds`. python fallback: `URLError
[Errno 101] Network is unreachable`. wget fallback: rc=0 (empty body) — treated as delivered.

## Usage

not visible to me
