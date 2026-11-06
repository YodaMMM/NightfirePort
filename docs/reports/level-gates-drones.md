# level-gates-drones — the drone-family gate sweep

Branch `s/level-gates-drones` (from `origin/master` 8e72343f8, merged — clean).

## The set

Every `Drone*`, `NDrone2_*`, `DroneFunc_*`, `DroneVision_*` function in ACTION.ELF
(SLUS-20579 NTSC-U, build e7ffdcb3) except the talk functions: **617 functions**
(the name-prefix list and the filename-grep list agree — verified by diff).

## What's done

1. **Session 1 (already on the branch):** the sweep read every function and found exactly
   one reached-without-port-code behaviour: the escort-abandonment monitor
   `NDrone2_ControlDTYPE_Zoe` 0x14b058 (DroneTypeSettings row 0x11's control fn
   [D 0x29aee8+204]; reached by the 07000008 VIP, dtype 12 + kind 0xc →
   `GetDTYPENEW` 0x14c938 case 0xc → +0xC5 = 0x11). While switch channel 1 is on and a
   player exists, `Vec_Dist3D(obj+0x30, glb_players+0x30) > 10.0` fails the escort every
   tick through `DroneFunc_SetMissionFailReason` 0x13e560's default arm (reason 0xf).
   Ported in `crates/nf-game/src/drone.rs`, tested red-on-old-code by
   `crates/nf-game/tests/zoe_abandon_live.rs` (both tests were red on the pre-fix code,
   per the session-1 commit record; both green now).

2. **Session 2 (this session):** the sweep turned into a checked-in table,
   `docs/port-map/drones.md` — one line per function:
   - **587 ported** — each row cites the port's code location, found by word-boundary
     search for the function's exact decompile address in `crates/*/src`, falling back to
     the function's base name (the port's fold style cites the decompile on the
     implementing function).
   - **30 not reached** — all `NDrone2_init_DMODE_*` DroneModeSettings row functions the
     shipped disc never dispatches (all placements carry parameter 5 = 100 → NEW path,
     which reads no row; MP bots and the one coder event take row 0). Proofs:
     `docs/systems/ai-dmode-inits.md`, `docs/port-map/unreached.md` §rows 1-22 and the
     2026-10-24 corrections (rows 23-30 dumped by address). Row 0 (`init_DMODE_Normal`
     → `init_DMODE_Defaults`) is reached and ported (`drone.rs:2394`).
   - **0 missing.**

   Method notes: the four functions whose only repo citations were test *comments*
   (`DroneAnim_CanDoAnimState` 0x13dfc8, `DroneFunc_DropGrenade` 0x14b158,
   `NDrone2_DSTATE_HeardNoise` 0x173be8, `DroneWeap_BurstDelay` 0x179ba8) were resolved
   by hand to their implementing code (`drone.rs:10505`, `drone.rs:5354`, the
   `HEARD_NOISE` 0xa2 arm at `drone.rs:7862`, `drone.rs:11167`) — a comment does not
   count. The 26 `NDrone2_DSTATE_Bot*` multiplayer states are ported as the lifted
   registry `mp/bots.rs::BOT_STATES` (each row carries the game address and the kind
   read off the .c — verified against the bodies: `BotGuardian` 0x174a28 really is just
   msg 1 → SetState 0xF9; `BotHeardNoise` 0x1752f8 returns `*msg == 0`), with real
   implementations for the Full-kind states in `BotBrain` and the `BotGlobal` 0x16E060
   router in `bot_global_dispatch` + `react_to_drone_alert_msg`.

## How it was verified

This session changed **no `crates/*/src` code** (docs only: the port-map table, WORKLOG).
Test runs on the merged branch, `NF_ISO` set, `--test-threads=1`, release:

- `nf-game`: the whole crate suite — **108 test binaries, 1095 passed, 0 failed**
  (includes `zoe_abandon_live` 2/2, `mp::bots` lib tests 27/27 with the
  `state_table_and_state_types` golden).
- `nf-formats` + `nf-audio`: **138 passed, 3 failed — only the 3 `disc_golden` tests**,
  the documented slim-ISO failures.
- `nf-view`: the walkreal for the task's namesake level,
  `walkreal_0700001b_live` — **17 passed, 0 failed, 2 ignored** (pre-existing,
  documented ignores). **The rest of the nf-view suite did not run in this sandbox:**
  the crate's 200+ test binaries out-built both the sandbox disk (linker died with a
  bus error at 94 % full) and the session clock even after pruning stale artifacts.
  Nothing in `crates/nf-view/tests/common/` was touched, and no `crates/*/src` code
  changed this session, so the un-run binaries' inputs are unchanged from master/merge
  state; still, the next session should finish the nf-view pass before any further
  code change.

## What's left

- The remaining nf-view test binaries (disk/time; see above). No code work left in the
  set: 587/587 reachable functions have port code at the cited locations, 30 are proven
  never dispatched, and the one gap the sweep found was fixed and tested in session 1.

## Live checks for the main agent

None required — the sweep changed no visible behaviour (the Zoe monitor fix is
gameplay-logic and its proof is the automated test; no HUD/render surface).

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (sandbox egress to ntfy.sh unreachable at ~19:10 UTC
2026-11-06):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-gates-drones: done — ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "... urlopen(...)"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-gates-drones: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → killed at the 30 s tool timeout with no output (connection hung).
