# drone-delete-turret-blast — the Drone_Delete turret arm

Branch `s/drone-delete-turret-blast`, cut from origin/master 576909ed6. Task: `Drone_Delete`
0x136ce8 (decomp ~90-96) — as a drone object is deleted, every class-0x36 turret whose
ext+0xb8 owner is that drone gets `obj+0xfe |= 1` and the
`Explode_Create(5.0, 5.0, 50.0, obj, obj+0x30, 0x2d6210, 0x6000052, 0xff, 0xff, 100, 0)`
blast at its own position.

## What the evidence says (the establish-first ask)

**The port's guards never deploy the turret, and no disc placement carries an 'R' weapon.**

- The deploy branch sits inside `DroneWeap_FireWeapon` 0x178f68 behind `ext+0xc5 == 0x1e`
  [M 00178f68:94 — the mode read precedes every gate in the block]. Mode 0x1e is the MP-bot
  row (the port's own census note: "Mode 0x1e is the bot row", drone.rs:7557); SP guards are
  never mode 0x1e, so **the whole guard-deploy branch (the 'R' check included) is MP-only**
  [M]. Its inputs live on BOT_vars+0x768 (the held weapon byte), and ext+0xd1c — the BOT_vars
  pointer — is written by `BOT_init` 0x1249e8:153 only, whose only callers are `BOT_respawn`
  0x124e38:76 and `MP_Start` 0x183a50:155 [M callers] — no SP path ever allocates it.
- The 'R' byte starts at `BOTWEAP_InitWeapon` 0x129e48:48 from the MP loadout row
  (`PickupMatrix + MPSettings…*10`, [M 0x129e48:43-48]) — never from a placement parameter.
  The MP loadout rows 1 and 3 do carry the Ronin 0x52 [D ACTION.ELF PickupMatrix @0x2b9100,
  the port's `PICKUP_MATRIX` const], so MP bots can hold it; the port's `BotBrain` firing
  pipeline (mp/bots.rs `handle_firing_armed`) models DoFiring's bullet path but has no deploy
  arm, so no port bot ever creates an owned turret.
- **[D] census (new test, green):** `crates/nf-formats/tests/gt_r_weapon_placement_census.rs`
  walks every 07* level on the retail ISO through the port's placement parser: **no drone
  placement (kind 0x0F) anywhere on the disc resolves its weapon (param 9) to 0x52** — and
  even if one did, the mode-0x1e gate makes the deploy branch unreachable for it.

Per the task's rule this is the "none does on disc" branch: reported here, with a **unit pin**
instead of a real-path walk pin (a real-path pin needs a guard that deploys — none exists in
the port or on disc).

## What landed

1. **The arm itself** — `GTurret::drone_delete_sweep(&mut [GTurret], deleted_owner) -> Vec<[f32;3]>`
   (gt.rs, beside `create_chain`/`player_has_died`): every turret whose `owner` is the deleted
   drone takes `delete_flag` (obj+0xfe |= 1) and returns its position for the blast. Verbatim
   decomp details kept: no state gate, **no already-flagged skip** (a turret flagged earlier in
   the same tick by GT_Create's MP sibling sweep blows again — the decompile's arm is
   unconditional [M 00136ce8:89-95], unlike `create_chain`'s port-side skip, noted in both docs).
   One port-fold gate: a PLACED turret's game owner is itself (GT_Create param 6 == 0 → own obj
   [M 0017ffa0:90-92]); the port stores that as `owner: 0, placement != usize::MAX`, and a
   self-owned turret never matches another object's deletion — so placed turrets are excluded
   even when the deleted drone folds to list index 0 (unit-tested).
2. **The wiring at the port's Drone_Delete fold points.** The game reaches `Drone_Delete`
   through obj+0xFE |= 1 → `control_delete_object` → control_funcs dispatch [M-dis
   0x133864-0x133888, 0x26F890]. The port folds that delete at five sites; all now raise
   `delete_me` (the flag itself): the FADE/FADE_FAST `gone` arm, the DEAD drawn-grace arm
   (already had it), `DeleteMe` 0xb4 (already had it), the GoToGoal prop-0x59 fold, the
   `PreDroneControl` tag-0xC arm [M 0x148db0:55-58 — that arm IS obj+0xfe |= 1], and the
   spawner `Action::Remove` (the game's per-drone obj+0xfe |= 1 [M 0x137e68:82,:267]). One
   consumer in `Drones::tick` (before the shared pending-blasts drain, so the blasts run the
   same tick) sweeps each newly-deleted drone's turrets once (new `turret_swept` latch field,
   own line at the struct tail) and queues `(pos, 5.0, 50.0, usize::MAX - 1, 0, 0x0600_0052,
   5.0, 0)` — the exact pending-blast form of the two existing turret-blast sites — through
   `spawn_blast`, which hurts bodies. A spawner re-add resets both flags (the game's fresh
   object). `DroneWeap_FireWeapon`'s deploy position math and `deploy_ronin` are untouched.
3. **The unit pin** — gt.rs `drone_delete_blows_only_the_dead_bot_s_turrets`: a bot-owned
   deployed turret is flagged and blasted at its own +0x30 (both of a pair), a turret owned by
   another object stays, a placed self-owned turret stays (also when the deleted drone folds to
   id 0), and an already-flagged turret is blown again (the no-skip rule).
4. **The [D] census test** — `gt_r_weapon_placement_census.rs` (asserts the empty result, so a
   future 'R' placement flips it loudly).

No decompile-backed test was touched, let alone weakened.

## How it was verified

- `cargo test -p nf-game --lib` (release, --test-threads=1): **684 passed / 0 failed / 5
  ignored** — includes all 21 gt:: tests (the new pin among them) and the compile of every
  struct literal (`--no-run` first, per lesson 9).
- The fast runner (`tools/test/suite.py`), foreground in chunks — this sandbox kills
  background jobs between tool calls, so the nohup start died twice; chunks with `--only`
  over the remaining binaries. **164/1132 binaries done: 701 passed / 3 failed / 46 ignored**;
  the only failures are the **3 `disc_golden` tests — the slim-ISO allowance** in the setup.
  Named binaries from the task and lesson 10, each whole:
  ai_live +37, ai_missing_states_live +9, ai_points_live +4, fix_ai_remainder_live +2,
  guard_ai_other4_live +4, stuck_guards_sweep +1, guard_reaction_live +1, guard_side_gates_live +4,
  guards_0c_live +1, door_guard_live +5, npc_doors_live +6/+0/i1, fix_guard_misc_live +6,
  drone_portcheck_live +5, guard_barks_live +4, guard_drops_flashbang_live +6,
  astronaut_guards_live +4, rope_guard_hitbox_live +4, gt_r_weapon_placement_census +1,
  ronin_live +9, gt_los_live +2, gun_turrets_live +3, mp_hud_real_live +4,
  disc_golden +11/−3 (the allowed three).
- The census needed `libasound2`'s runtime .so added to the user ALSA prefix (the dev package
  alone leaves `-lasound` unresolved); noted here because the next sandbox hits the same wall.

## What's left

- **The remaining ~968 suite binaries** (the nf-view level walks) did not fit the window; the
  next message can resume the chunked runs from `s/drone-delete-turret-blast` (harvest state:
  701 passed / 3 failed / 46 ignored over 164 binaries, all green but the allowed disc_golden).
- **The deploy layer itself** (the task's "otherwise" branch, now proven MP-only): a future MP
  task can land `BotBrain`'s 'R' deploy arm (FireWeapon 0x178f68:122-138: `GT_DeployMiniGun`
  at (x+sin yaw, y, z+sin(yaw+π/2)), `BOTWEAP_decrRounds(…, 1000)` + `BOTSTATE_changeWeapon(…,
  0, 1, 0)`) — the delete-side sweep, the MP sibling chain and the pending-blast plumbing are
  already in place and pinned; a deploy would only need the turret creation with the bot's
  list index as owner. The real-path pin (guard deploys → deleted → turret blasts; other-owner
  turret stays) belongs to that task, where a real deploy exists to walk.

## Live checks for the main agent

None required — nothing visible changed (no draw path touched; the sweep is unreachable in
real play on retail data, per the census).

## Usage

not visible to me
