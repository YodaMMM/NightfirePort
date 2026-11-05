## Your task: `mp-bot-body-sounds`, MP bots make their jump and head-bump sounds

A bot's movement core queues the same jump launch and head-bump sounds as a human's (`Player_HandleJump` 0x1916b0 through `Bond::sfx_out`), but crates/nf-game/src/mp/bots.rs:3243 clears them every tick, while scene.rs drains the remote humans' bodies into `mp_fx.body_sounds`. Read docs/reports/mp-bot-voices.md first. Confirm from the decompile that bots reach these sounds [M], then drain the bots' queue the same way so each plays at the bot's position, on every machine, with no change to the shared Rand stream or the state hash. If the decompile shows a bot does not reach one of them, keep that one silent and cite the address. Pin it through a real match with a bot that jumps. Run the mp_*_live binaries, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-body-sounds`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-05T19:58Z

---
## Report (done, 2026-11-05T21:20Z)


---
## Report from the worker (done, 2026-11-05T21:20Z)

# mp-bot-body-sounds — MP bots make NO jump or head-bump sound (the decompile says so)

Branch `s/mp-bot-body-sounds` (cut from origin/master aa1218626). Tags: [M] code (function +
address, read in `work/decomp/ACTION.ELF/` and the raw ELF), [D] disc, [L] live PCSX2
measurement, [S] suggested. Read `docs/reports/mp-bot-voices.md` first (its "What's left" named
this exact gap).

## The task's premise inverted: the bots reach NEITHER sound

The ask was to drain the bots' `sfx_out` queue like the humans' — unless the decompile shows a
bot does not reach the sounds. It does not, for either sound. The full census:

1. **Only `Player_HandleJump` 0x1916b0 plays the sounds.** The launch 0xA3 [M :184 — disasm
   `addiu a0,zero,0xa3` at 0x1918d8 → `Sound_Play3D` 0x1dd548 at 0x191924] and the head bump
   0x5FB/0xA3 [M :195-198 — 0x1919e4/0x191a14, the level-tag 0xD/0xE arm]. A whole-binary
   immediate scan of ALL 74 `Sound_Play3D` call sites and ALL `Sound_Play` 0x1dd880 call sites
   (recovering the id immediate into each argument register) finds no other producer of id 163
   or 1531 — and none in SLUS_205.79 or DRIVING.ELF either.
2. **Its only caller is `Player_SSWalk`** (single `jal` at 0x192474) [M], whose only caller is
   `Player_Update`'s substate-0 arm (single `jal` at 0x193964, the `obj+0xf6` switch) [M].
   `Player_Update` 0x193600 itself has zero direct callers — it runs only as an object's
   registered movement handler.
3. **`Player_Update` is the movement handler of object classes 3 and 0x12 ONLY.** The dispatch
   is `control_funcs` @0x26f890 read as `control_funcs[obj+0xff · 0xc]`
   (`control_movement_object_handler` 0x1336f8:62-63) [M]: class 3 → +0x24 = 0x193600, class
   0x12 (18) → +0xd8 = 0x193600. Classes 2 and 0x11 → `Drone_Control` 0x139268 (+0x18/+0xcc).
4. **The MP bots' objects are class-2 drones, and never become class 3/0x12.** `MP_Start`
   0x183a50 creates each bot with `BOT_init(slot, spawn, …, obj = NULL, …)` [M]; `BOT_init`
   0x1249e8 calls `Drone_Create` 0x136be0 (a zeroed DIVars) → `NDrone2_CreateObj` 0x14d7c8
   writes `obj+0xff = 2` [M]. A whole-binary scan of every `sb …,0xff(…)` store cites every
   class write in ACTION.ELF: `= 3` only at `Player_Init` 0x1912c4 (the humans; MP_Start's
   human loop) and `MP_ReSpawn` 0x185474+0x185578 (`addiu s7,zero,3` … `sb s7,0xff`) — and
   `MP_ReSpawn`'s class-3 arm runs only for objects ALREADY class 3/0x12 (a bot's class-2
   object takes the reposition-only arm 0x185368:69-76); `= 0x12` only at
   `Player_CheckForDeath` 0x1909ec (the human corpse); `= 0x11` at `BOT_respawn` 0x124f7c and
   `MP_CheckForEndCondition` 0x1845ac (the disabled bot shell), which also dispatches
   `Drone_Control`.
5. So a bot's body is moved by `Drone_Control` → the NDrone2 anim/root-motion mover — the same
   reading the port's own `BotBrain::body` doc already carries ("stands in for the drone mover
   … [S]"). A bot never runs `Player_Update` case 0 → never `Player_SSWalk` → never
   `Player_HandleJump` → **neither the jump launch nor the head bump plays for a bot, on any
   machine.** The humans' classes (3 alive, 0x12 corpse) DO run it — the port's existing
   drains are right: the local body into `drones.audio` (scene.rs) and the remote bodies into
   `mp_fx.body_sounds` → `mpfx_play3d` (mpfx.rs, slot usize::MAX).

## What changed in the port

- **No behaviour change.** `walk_body`'s `self.body.sfx_out.clear()` (bots.rs, was line 3243)
  IS the game-accurate silence. It now carries the full citation above; the task's "no change
  to the shared Rand stream or the state hash" holds trivially (nothing but the comment and
  tests moved).
- **Pin 1 — a bot that jumps drops the sounds** (nf-game, `brain_tests::
  a_bot_that_jumps_plays_no_launch_sound`): the REAL `walk_body` path (`BotBrain::step`, the
  call the sim makes every tick) with a full-span wall wedge and a pickup goal beyond it — the
  bot's own `pick_goal` → `steer_to` (nav-less straight steer) pushes into the wall, the stuck
  watch (`walk_body`'s 45-tick < 0.4 escape) fires the body's jump, and the test asserts the
  queue is empty after EVERY step while the launch is observed (a rising airborne body — the
  only upward-velocity write in the movement core is the launch impulse, −GRAVITY·0.4 [M
  player.rs]; the tick's gravity integration runs after it, so the post-step sample is already
  below 3.92).
- **Pin 2 — match-long silence** (nf-view, `mp_bot_body_sounds_live`): the Quick Start shape
  crowded to the perf16 roster (14 bots, arena 07000023, 3600 ticks): no 0xA3/0x5FB reaches
  `mp_fx.started` (any slot, any channel) and every bot's `sfx_out` is empty after each step.
  3185 airborne bot samples observed (falls — no launches; see the honesty note below).

### Honesty note on the live jump

The live match's bots never wedge on their own: their route follower rounds every obstacle, so
the stuck watch never fires in a natural match (0 launches across every configuration tried:
3 bots, 14 bots, corner teleports, a pickup lure beyond a wall — each probed with
`CollisionWorld::segment`). The jump-drop proof is therefore the unit pin, which drives the
same real `walk_body`/`step` code with the walk request the AI would produce; the live pin
holds the match-long silence. A bot that jumps in the port is a stuck-watch event, and when it
happens the sound is dropped — pinned, not hoped.

## Runs (this sandbox, release, `--test-threads=1`, NF_SILENT=1, NF_ISO set)

- **nf-game lib: 683 passed / 0 failed** (5 ignored — includes the new wedge pin).
- **nf-game integration (the bots/sound set)**: `mp_arenas_live` 1/0, `sound_coverage_live`
  5/0 (the jump-sound law on a human body — the pin's sister), `find_opponent_live` 3/0.
- **nf-view `mp_*`: 38 of 40 binaries green** (0 failures anywhere):

| binary | passed | binary | passed |
|---|---|---|---|
| mp_bot_body_sounds_live (new) | 1 | mp_16_slots_live | 3 |
| mp_world_sync_live | 8 | mp_body_anim_live | 2 |
| mp_bots_live | 3 | mp_clock_gate_live | 1 |
| mp_bots_moving_live | 3 | mp_combat_weapons_live | 11 |
| mp_core_live | 5 | mp_death_effects_live | 3 |
| mp_bot_voices_live | 3 | mp_death_hands_online | 2 |
| mp_remote_fx_live | 2 | mp_pickup_take_live | 2 |
| mp_bodies_live | 3 | mp_pickups_visible_live | 2 |
| mp_bot_weapons_live | 2 | mp_respawn_ammo_live | 3 |
| mp_bot_objectives_live | 10 | mp_site_damage_offline_live | 4 |
| mp_bot_armour_online_live | 1 | mp_shot_effects_live | 2 |
| mp_scenarios_live | 19 | mp_shot_gate_live | 5 |
| mp_status_strings_live | 1 | mp_match_end_live | 2 |
| mp_defaults_setup | 12 | mp_host_match_live | 2 |
| mp_hud_16_live | 2 | mp_hud_real_live | 4 |
| mp_net_slots_post_move_live | 2 | mp_perf_16_live | 2 |
| mp_lobby_one_screen | 7 | mp_menu_flow | 11 |
| mp_menu_lists_live | 5 | mp_net_lobby | 11 |
| mp_screens | 5 | mp_debriefing_flow | 6 |

- **`mp_world_sync_live` 8/8** — the per-tick state-hash equality (incl.
  `skyrail_with_bots_hashes_agree`, 2 bots online, two machines) is intact with this branch.
- **The fast runner (`tools/test/suite.py`) could not run to completion here**: both nohup
  launches were reaped between tool calls with `suite.out` still empty (the same behaviour the
  mp-bot-voices report documented), and the foreground prebuild of nf-game + nf-view exceeds
  the 10-minute tool limit even incremental (two 10-minute chunks). The required set above was
  run per-binary instead, on the final tree (branch == master-merged: master had not moved
  when re-checked after the work — `git log origin/master` still at aa1218626).
- **Not run**: `mp_online_failures` and `mp_stress_explosives_live` — both die by SIGKILL (OOM)
  on this 4 GB sandbox per the mp-bot-voices report (verified pre-existing on a pristine master
  worktree there); not re-run here for time. The non-mp nf-view/nf-format/nf-audio binaries
  were not run: the touched surface (bots.rs's `walk_body` comment + two new tests) has no
  consumer outside the mp set above.

## What's left

- The two OOM binaries above on a larger-memory machine.
- The full `suite.py` pass on a sandbox where background jobs survive.
- Nothing behavioural: the port's bot sound law is settled, cited, and pinned.

## Live checks for the main agent

1. **A bot's jump is silent** [M 0x1916b0 is class-3/0x12-only]: in PCSX2, an MP match with a
   bot; get a bot to jump (wedging it against a wall is the reliable way). Watch `Sound_Play3D`
   id 0xA3 (163): expected — the HUMAN's jump plays it, the bot's jump plays NOTHING (no 163,
   no 1531) at the bot's position.
2. **The bot's mover** [M control_funcs 0x26f890]: breakpoint `Player_Update` 0x193600 in a
   bot-populated match and check the `obj+0xff` class byte of every object it runs: expected —
   only 3/0x12 (the humans); never the bots' 2/0x11.

## Usage

Not visible to me. This sandbox exposes no token, credit or context-window figures for the
session: there is no environment variable, file or tool output in this session that reported
any such numbers (checked: env, /proc, the shell banner). Not estimated.
