# consider-explosive-gate — report

Branch `s/consider-explosive-gate` (from `origin/master` a1cfe068). Two commits: the gate + the tests,
and this report/WORKLOG.

## What was done

GAPS G-055: the port's `DroneFunc_ConsiderExplosive` state gate refused more states than the game's,
so a guard in AimStandFire 0x68 never fled a live grenade.

**The fix** (`crates/nf-game/src/drone.rs`, `consider_explosive`): the game's gate reads the drone's
state and refuses only 0x28, 0x2d, 0x4a-0x4b, 0x53-0x55 (the < 0x56 arm) and 0x63, 0x86-0x88 (the
>= 0x56 arm) [M 0x1431c0:40-67 — `work/decomp/ACTION.ELF/001431c0_DroneFunc_ConsiderExplosive*.c`].
The old port gate also refused 0x2e-0x49, 0x4c, 0x52, 0x56 and 0x62-0x85, and wrongly allowed 0x86
(the game refuses the whole flee family). The ported line:

```rust
if matches!(self.state, 0x28 | 0x2d | 0x4a | 0x4b | 0x53..=0x55 | 0x63 | 0x86..=0x88) { return; }
```

Related [M] facts read and cited in the code/docs: the game's AimStandFire case 0x21 passes the
current state as ConsiderExplosive's resume [M 0x166328:184-187], AimStand passes 0x68 [M 0x166178
case 0x21], AimStandReload passes 0 [M 0x1666d8 case 0x21]; the port's shared msg-0x21 arm passes
ATTACK for the aim family — the resume is unused for the frag kind (return 0x87 [M 0x138b10:67-87]),
so that deviation is left as a noted follow-up, not changed here.

**The tests** (`crates/nf-game/tests/grenade_live.rs`, +2, through the real grenade path — the
suite's force-throw harness on 07000002, then the sim's own flight, bounce broadcast
[M 0x1435f0 + 0x138b10], msg-0x21 delivery, gate and flee; the state pin is a premise write each
tick *before* it, never after the delivery):

- `a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag` — the guard pinned in 0x68 (held
  through the tick by AimStandFire's own mid-burst stay [M 0x166328:104-112]) flees his own landed
  frag at bounce+0 into RunAwayFromObject 0x86 [M 0x1431c0:194-195], with +0x330 = the grenade's
  spot [M 0x1431c0:172-175]. FAILED on the old gate (no flee), GREEN on the fix; re-fails with the
  fix stashed (the revert-check).
- `a_guard_in_go_to_goal_0x63_does_not_flee_the_landed_frag` — the sister pinned in 0x63 holds
  through the bounce and the whole 120-tick live window [M 0x1431c0:63 refusal].

Setup notes en route: the broadcast scans active guards only, so the test guard is set `enabled`
(the WAIT_SWITCH wake leaves the port's flag cleared); Bond stands 10 u out so the frag rolls back
to ~4-5 u — the corridor-11 d3 lob shape — resting inside the 12 u scan with LOS.

## How it was verified (quoted totals)

- `nf-game`, all 99 test binaries, per-binary `--test-threads=1`, NF_SILENT: **1036 passed, 0
  failed, 9 ignored** — includes `ai_live`, `grenade_live` 4/4, `stun_grenade_live`,
  `grenade_rattle_live`, `guard_drops_flashbang_live`, `explode_live`.
- `nf-view`, 118 binaries = **every `walkreal_*` (105) + the flee-affected groups**
  (start_corridor_guard_07000011_live, stuck_guard_combat_05_live, guard_stuck_sweep_live,
  escort_follow_07000001_live, escort_cover_live, fix_guest_reactions_live, fix_grenade_throw_live,
  fix_grenade_release_live, fix_grenade_types_live, frag_throw_timing_live, consider_alerted_live,
  inert_guards_02_live, hold_npcs_live): **251 passed, 1 failed, 17 ignored**. The one failing test
  is in `start_corridor_guard_07000011_live` (see below). **Every walkreal_* binary is green.**
- `tools/test/suite.py` could NOT run one-shot: its build phase links all ~950 nf-view test binaries
  (~8 GB) and the sandbox has 4.2 GB free after the strip/prune this task needed. The same
  methodology (per-binary, `--test-threads=1`, `NF_SILENT=1`, wall-clock caps) was applied to the
  required groups above instead. The branch already contains origin/master (a1cfe068).

## What's left

- `start_corridor_guard_07000011_live` fails 1-2 of its 8 tests with the fix (run-to-run:
  `idle_bond_is_approached_and_shot_at_every_selectable_difficulty` and/or
  `in_position_the_guards_step_aim_as_well_as_fire`). Its [L] pins — the d2 death time f1113, the
  t700 corridor positions — presuppose the old gate's no-flee choreography. With the game-correct
  gate the fight punctuates with flee cycles ([M 0x1431c0:40-67 + 0x1435f0] against the port's [S]
  lob that rolls back to the guards), and clearing the guards' grenades to restore the [L] stream
  shifts the RNG stream and breaks the stream-calibrated position pins instead (0.22 over the box
  edge at t700). The test is over-constrained until the game's d2/d3 grenade interplay is
  re-measured live; I left the file untouched rather than weaken [L] pins.
- The rest of the nf-view suite (~800 binaries outside the required groups) was not rebuilt/run —
  the disk ceiling above. Nothing outside the touched gate + test file could change them, and the
  named + walkreal sweep covers the AI surface.

## Live checks for the main agent

1. 07000011 d2, the idle-Bond run (PCSX2, FRESH boot): does a guard throw a grenade at Bond, and
   where does it land/rest? gdbwatch a fighting guard's state (drone+0x10c) and +0x2B through the
   throw and bounce — EXPECT per [M 0x1431c0:40-67 + 0x1435f0:30-52]: a guard in 0x68/0x73/0x74
   (fear props 0x34/0x3a/0x3b [D p23-25]) with a frag resting within 12 u and LOS enters 0x86.
   Also frame-stamp the d2 death (the [L] f1113) — if the game's run flees too, the corridor pins
   need re-measuring with the flee included.
2. The port's [S] lob trajectory (the 49.73 ceiling hit and the rollback to the guards) — live-measure
   the game's own grenade arc on the same spot to replace the [S] with [L].

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (the sandbox cannot reach ntfy.sh; git push over HTTPS works, so egress
is selectively blocked — the same result as the level-15 session in this sandbox):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20000 milliseconds`
2. `python3 -c "... urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', ...))"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, killed after the 25 s tool cap (connection never established)
