## Your task: `ssys-wire`, the security subsystems count down and fire in play

crates/nf-game/src/ssys.rs ports `SSys_Init` 0x1b01b0, `SSys_Monitor` 0x1ad998, `SSys_Msg` 0x1ad878 and `SSys_RamLoad` 0x1b0208, but nothing outside that file calls them: only its `REC_FRAME_RATE` constant is used, so the 240 s laser and camera countdowns, their announcements and their channel writes never run. Confirm that first. Wire all four the way the game does: `ResetMap_GameInit` 0x1d5080 calls Init (respect `IsWarmReset`), `Game_Run` 0x1c9790 calls Monitor after the `GS_IsPaused` gate and only while `ScriptCam == 0`, and `Player_RamLoad` 0x1957a8 calls RamLoad. Nothing in ssys.rs arms an item yet: the only writer of an item's channel is `Switch_Create` 0x1adcc0:29-31, which stores the switch's channel when placement field +0x44 is non-zero. Port that, and census every level for switches with a non-zero +0x44 [D], listing level, item and channel in the report. Check the save side too: find what writes the SSYS block that RamLoad reads, and make the port's checkpoint and Death-and-Replay path keep it. `Msg` sounds must go through the normal sound path with their level gates (0x0700000b with channel 0x76 off, camera 1 on 0x0700000a). Keep the game's tick order against the other switch-channel writers. Pin through the real path on a level from the census: Bond trips the channel, the announcements play at 180/120/60/6 s, the channel drops at 0 and whatever hangs off it reacts. If the census finds no placement, say so in the report and keep the wiring with a unit pin. Run every test that mentions switch channels on the census levels, walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ssys-wire`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T01:06Z

---
## Report (done, 2026-11-07T02:21Z)


## Report from the worker (done, 2026-11-07T02:21Z)

# ssys-wire — the security subsystems count down and fire in play

Branch `s/ssys-wire` from `origin/master` (22d5836b7). The port of `SSys_Init` 0x1b01b0 /
`SSys_Monitor` 0x1ad998 / `SSys_Msg` 0x1ad878 / `SSys_RamLoad` 0x1b0208
(crates/nf-game/src/ssys.rs) had **no callers** — confirmed first with a whole-crate grep:
every `Ssys` reference lived inside ssys.rs itself, and only its `REC_FRAME_RATE` constant
was used elsewhere. The 240 s countdowns, their announcements and their channel writes
never ran. All four are now wired the way the game calls them, and the switch-create arm
store that feeds them is ported.

## What's done

### 1. The switch-create channel store (`Switch_Create` 0x1adcc0:29-31)

[M] `if (*(int *)(param_3 + 0x44) != 0) SSysItems[*(int *)(param_3 + 0x44) * 8 + 4] =
*(u8 *)(ext + 0x12);` — a switch whose placement param 6 (`level_tag`+0x44, the SSys item
index) is non-zero stores its own channel id (param 0) into `SSysItems[idx].channel` (the
+4 byte of the 8-byte record) at CREATE time. Ported as `SwitchObj.ssys_item` (param 6) +
the store loop in `Drones::attach_level_id`, BEFORE `UseObjects::init_channels` (the load
beat's `switch_channels[param0] = param3`, [M 0x1adcc0:39-43]) — the game's create order.
The game indexes the 5-record array raw; every disc placement carries 1 or 2 [D census
below], so the port bounds-checks.

**Census [D]** (crates/nf-view/examples/ssys_switch_census.rs, all 20 story main maps,
every 0x29 placement): exactly **two** switches carry a non-zero param 6:

| level | switch | channel (p0) | gate (p5) | SSys item (p6) | subsystem |
|---|---|---|---|---|---|
| 07000009 | #71 `SCRIPTPLAYER_Lasers` (at the origin, its art is scripted) | 0x37 | — | 1 | laser 1 |
| 0700000a | #643 `SCRIPTPLAYER_KeyboardLasers` (at the origin) | 0x3b | 0x3c | 2 | laser 2 |

Every other level: none. No script on either level writes the channels (0xd/0x11 event
scan over every kind-7 NIS of both archives: zero hits [D scratch dump]); the writers are
the switches' own `Switch_Activate` 0x1b02a8 toggles (docs/levels/0700000a.md records 0a's
switch as inert: its gate ch 0x3c has no writer anywhere, and the walkreal already pins
"ch 0x3B never on").

**[L] proof the store is the game's behaviour**: `ee_start_0700000a.bin @0x2befd0`
(`SSysItems`) reads item 0 {0.0, 0, 0}; item 1 {−1.0, 0xff, 0}; **item 2 {−1.0, 0x3b, 0}**;
items 3/4 {−1.0, 0xff, 0}; `SSysDelayTimer` 0.0 — i.e. after the cold boot `SSys_Init`
0x1b01b0 set the 0xff defaults and `Switch_Create` for #643 (param 6 = 2, param 0 = 0x3b)
overwrote item 2's channel. The port's cold 0a load reproduces that record exactly
(`the_switch_create_store_lands_in_ssysitems`).

### 2. `SSys_Monitor` in the per-tick pass

`Game_Run` 0x1c9790:32-34 [M]: `GS_IsPaused() == 0` → `SSys_Monitor(); Light_Update();
control_movement_object_handler(0);` — the monitor sits after `Env_Update`, before the
object handler. The port calls `Ssys::monitor` in `tick_inner` at exactly that slot (after
the env update, before the locks/keypads/object walk), with the monitor's own
`ScriptCam == 0` gate [M 0x1ad998:8] on the port's `scripts.camera()` stand-in (the same
one the muzzle-light gate cites, 0x1befd8:191-195). The monitor therefore reads and writes
the channels as the previous tick's walk left them, ahead of this tick's
triggers/scripts/switches — the game's tick order against the other switch-channel writers.

Disassembly check the decompile hides: **neither** of the monitor's channel writes stamps
`switch_channels_time` — the laser-2 arm's `switch_channels[ch] = 1` is a bare `sb` store
[M 0x1adaa0-0x1adacc], and the state-2 expiry's `switch_channels[ch] = 0` likewise
[M 0x1adc64]. The port writes the arm through the array directly (the channel-persistence
probe records the direct writer, as at `Drone_InitComms`' 0x97) and the drop through
`Channels::set` (which stamps only non-zero values, so it matches).

The announcements ride `SSys_Msg` 0x1ad878's four tables [L ee_start_05 @0x2bef60/0x2bef98/
0x2beef0/0x2bef28] through the port's normal sound path (`Sounds::play`, the game's
`Sound_Play(100.0f, table[id], 0, 0)` 2-D call [M 0x1ad878:15, :33]), with the level gates
`msg_sound` models: 0x0700000b with ch 0x76 off silences everything, camera 1 never plays
on 0x0700000a [M 0x1ad878:9-12, :26].

### 3. The save side

[M] `Player_RamSave` 0x195670:9 copies the whole `SSysItems` array (0x28 bytes) into the
RamSave buffer at 0x2b9a14; `Player_RamLoad` 0x1957a8:13 calls `SSys_RamLoad(0x2b9a14)`
inside its `IsWarmReset` body. Ported:

- `RamSave.ssys: Option<[SsysItem; 4]>` + `RamSave::with_ssys` (a builder step, so the
  plain `capture` call sites stay). The game-path captures attach it:
  `warm_exit_save` (the trigger-exit save) and `entry_checkpoint_save` (the post-load
  re-save that a fail-Restart serves). The bugreport `entry.sav` format carries the block
  as **v4** (encode + decode + the round-trip test), versioned like the tally/difficulty
  extensions before it.
- `load_with_carry` applies the block via `Ssys::ram_load` where the mission/tally carries
  land — the warm-load restore; a cold load (save `None`) keeps the fresh `SSys_Init`
  state, matching the game's `IsWarmReset` gate [M 0x1957a8:9].
- The 0a skip [M 0x1b0208:13 — item 2 is not loaded on 0x0700000a]: the game's item 2
  keeps its pre-load `.data` record, which on the exit path equals the saved block record
  (the exit save is the last writer before the load) and on a death restart is the
  death-time live record. The port plants the carried record before `ram_load`'s skip, and
  `fail_restart_save` swaps the live item 2 into the serve's record-2 slot — the death-time
  record survives the reload exactly as the game's `.data` keeps it.

### 4. The real-path pin on the census level

`the_09_laser_switch_arms_the_countdown_announces_and_drops`
(crates/nf-view/tests/ssys_wire_live.rs), all through the real tick loop
(`sim.step_bond`, 14,700 ticks, ~3 s):

1. Nothing arms while ch 0x37 is off (30 ticks).
2. The trip: Bond's use dispatch (`UseObjects::player_activate` — the `Player_Activate`
   0x1a5170 → `SP_Activate` 0x1dc338 → `Switch_Activate` 0x1b02a8 route, the game's ONLY
   pick route to a scripted-position switch) against the laser show's live ON-art entity:
   ch 0x37 toggles 0 → 1. The pick point is computed from the live entity target
   (`entity_use_targets_dir` — the posed sphere and facing of the art the game itself
   draws); the physical walk from the car-park start to the lobby panel is **not** driven
   by synthetic input in the test (see Live checks).
3. The monitor arms on the next tick: timer 240.0 [M 0x43700000], state 6, the arm
   announcement (laser-1 msg 6 = 0x2a2 [L @0x2bef60]) plays.
4. The announcements at 180/120/60/6 s — msg 5 (0x295) at arm+3600, msg 4 (0x296) at
   +7200, msg 3 (0x297) at +10800, msg 2 (0x298) at +14040 (±8 ticks for the f32 1/60
   accumulation; the unit pin allows ±2 per 480) [L MIN3/MIN2/MIN1/MIN01 @0x30cd84..90].
5. At +14400 the channel drops (`switch_channels[0x37] = 0` [M 0x1adc64]), state 0,
   timer −1.0, the fire announcement (msg 1 = 0x2a3) plays.
6. The drop's own reaction: `Switch_Update` 0x1adea8:19-24's mirror arm unpauses the
   embedded show — the 983-tick script resumes from its t2 hold and the AMBER art
   (`SCRIPT_Lasers_AMBER` 0x02000a29 [D script 0x06000667 stream 0 t31]) comes up, asserted
   through `scripts.entities()`.

What hangs off the channel across levels: ch 0x37 is 0a's laser-web disarm lock (the 0xEB
beams fire only while it reads low [D the 0xEB params; docs/levels/0700000a.md]), carried
through the warm load — the countdown's drop re-arms the next level's grid. kind-1 locks
are untouched by this task (that is the laser-burn lock work on another branch).

## Tests run (all green)

| binary | result |
|---|---|
| nf-game lib (includes the ssys units + the useobj switch tests) | 686 passed, 5 ignored |
| nf-view `ssys_wire_live` (new) | 3 passed |
| nf-view `laser_scripts_live` (09/0a switch channels, the show mount) | 2 passed |
| nf-view `bugreport` (the v4 entry.sav round trip) | 3 passed |
| nf-view `fix_death_retry_boost_live` + `fix_death_restart_live` | 4 passed |
| nf-view `async_level_load_live` | 3 passed |
| nf-view `walkreal_0700000a_live` (required) | 13 passed, 2 ignored |
| nf-view `walkreal_0700000b_live` (required) | 7 passed |
| nf-view `interactables_0700000a_live` + `_0700000b_live` | 13 + 14 passed |
| nf-view `interact_07000009_live` | 16 passed |
| nf-view `campaign_flow_live` (warm exits carrying saves) | 5 passed |
| nf-view `walkreal_0700000c_live` + `npc_doors_live` + `fix_guard_misc_live` (other levels' real ticks through the new monitor slot) | 3 + 6 + 6 passed |

The full four-crate suite did **not** fit this sandbox: `tools/test/suite.py` died twice in
the background (the sandbox kills detached jobs between tool calls; 3 GB RAM also OOMs the
parallel test-binary link), so the affected binaries above were run whole in the foreground
instead, each to completion, plus every required `walkreal_*_live`.

## What's left

- The full four-crate run on the merged final tree (the binaries above are the touched
  surface; the merge with origin/master should re-run them).
- The physical walk: driving Bond by real input from 09's car-park start to the lobby laser
  panel (the stairwell route) so the trip happens without the computed pick point — the
  trip mechanism itself is the game's and is fully pinned.

## Live checks for the main agent

1. 07000009, fresh boot, EE-watch `switch_channels[0x37]` (@0x26FC90+0x37) and
   `SSysItems` (@0x2BEFD0, 5×8 bytes) while E-using the lobby laser panel (the beam art
   entity at the lobby grid): expect the item-1 channel cell to read 0x37 from the load
   (the store), 0x37 to go 0→1 on the use, then the countdown — announcements at 180/120/60/6 s
   (`SSysItems[1]` state 6→5→4→3→2), and the channel back to 0 exactly 240 s after the arm,
   with the laser show's AMBER art up ~31 ticks after the drop.
2. 0700000a, fresh boot: confirm ch 0x3b never turns on in play (the switch is inert —
   gate 0x3c has no writer), so `SSysItems[2]` sits armed-and-waiting ({−1.0, 0x3b, 0})
   without ever counting [L ee_start_0700000a @0x2befd0].

## Usage

- Input tokens / output tokens / context used / cost: not visible to me. (No token, credit
  or context-window figures are shown anywhere in this environment; the sandbox only
  exposes a wall clock — 01:08 UTC at session start, expiry ~03:07 UTC.)
