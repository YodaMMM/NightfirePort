# level-06-castle-fire — 07000006 held and repeated fire, then the castle interior walk

Branch: `s/level-06-castle-fire` (from `origin/master` @ 26543436, merged — master had not moved).

## What the task asked

The two `walkreal_07000006_live` tests were ignored on a measured player-weapon fault: the first
press spawns the round but turns the view −1.38 rad from the aimed yaw, and every later press
spawns nothing (12 presses → 1 bullet). The task: reproduce both faults with real input on
07000006, find the cause in the fire path [M `Player_SetFiringAnim` 0x1a3c70,
`Player_WeaponFiring` 0x1a5460, `Player_SetWeaponAnimObj` 0x1a2e38] against the port's
`Gun::update_with` and the fire-script/state handling in `crates/nf-view/src/scene.rs`, fix and
pin it, then un-ignore the two 06 walks.

## Finding: the fire-path fault does not reproduce on current master

I reproduced the walkreal's exact fight conditions through the real input path — the same carry,
the same walked lane, the same `Sim::step_bond` trigger path — at **both** fight sites on the
route:

1. **The guardhouse courtyard** (the idle guard #438 at (27.51, 39.18, −10.19) [D census; he is
   `drones.list[2]`, 10 hp, state 0x4 at load], the lane to door #47).
2. **The searchlight-courtyard mouth** (past doors #47/#59, after the entry plates #525/#526
   stage SP #808's boot NIS — ch 0x52 measured on; the fight the walkreal's `fight_through`
   runs), fought against the nearest courtyard guard (dist 10.2 at the mouth).

At both sites, through the real fire path (`step_bond` trigger → `Gun::update_with` → the shot
block), on the current master code:

- **Every press fires.** 12 real trigger presses ≥ 8 ticks apart → 12 rounds, including a
  mid-battery reload done through the real released-trigger arm (the auto-reload correctly waits
  for the playing fire script to end — the game's `0xf4 == 0` gate [M 0x1a5460:447-450]).
- **The press tick turns the body nothing** (the test presses with zero steer input; the measured
  per-press Δyaw is < 0.02 — the walkreal's −1.38 never appears). This matches the decompile: the
  fire branch spends the round and spawns the bullet from the eye along the view [M 0x1a5460:688-693],
  `Player_SetFiringAnim` 0x1a3c70 only queues the view script, case 9 of 0x1a2e38 is the view
  machine — none of them writes the body yaw.
- **Held fire with the semi-automatic P2K is exactly 1 round** in 90 held ticks (the pull re-arms
  on the fresh-press edge only [M 0x1a5460:601-655; L the P2K record spp 1 / interval 8]).

The old measurement (commit 7aed6dc7's probe logs, not committed) predates the fists fix in
`switch_to_armed` and whatever else moved since; per the task's own discipline the honest outcome
is: the repro tests now pin the proven behaviour, and the doc comment on the walkreal test is
rewritten with the new evidence instead of asserting a fault the port does not have.

## What was added

- **`crates/nf-view/tests/fix_06_fire_live.rs`** (new, opt-in via NF_ISO, all green):
  - `twelve_presses_spawn_twelve_rounds_and_nothing_turns_the_body` — walks the truck-ride lane to
    the guardhouse court (the walkreal's own legs), arms the carried P2K [L level_start: 7+41],
    aims with the real view steer, and presses 12 times 20 ticks apart (with the real reload when
    the 7-round clip runs dry). Pins: each press fires on its press tick (`shot_tick` + the
    in-flight bullet count) and no press turns the body [M 0x1a5460; L P2K spp 1 / interval 8].
  - `held_fire_with_the_semi_auto_is_one_round` — 90 held ticks through `tick_fire` → exactly 1
    round [M 0x1a5460:601-655; L P2K spp 1].
  - `the_searchlight_courtyard_fight_fires_every_press` — the full walked route to the mouth (the
    guard #438 dies through the real fire path, doors #47/#59 E'd, the boot NIS staged and
    waited out), then 6 presses → 6 rounds, no press-tick yaw turn.
- **`crates/nf-game/src/gun.rs`** — `held_automatic_fires_at_the_record_rate`: an automatic held
  37 ticks fires at ticks 0, 6, 12, 18, 24, 30, 36 [M 0x1a5460:601-655; L weapon 0x12's record:
  spp 999 (u16 at +0x28) / interval 6 (tick count at +0x40), measured from
  `work/ee_start_05.bin` weapon_data @ 0x2bf150, stride 0x10C].
- **`crates/nf-view/tests/walkreal_07000006_live.rs`** — route and docs:
  - the guardhouse fight (`fight_through`) on the lane to door #47 — the guard dies through the
    real fire path;
  - the explicit **through-door #59 waypoint** — the auto-closing doorway is a nav blind spot and
    without it the mouth leg aims through the guardroom's west wall (measured: a 6000-tick wedge
    at (28.54, 41.03, −19.05) with a wall segment 0.027 into the eye ray);
  - both tests stay `#[ignore]` with rewritten, evidence-carrying doc comments (see "What's left").

## How it was verified

- The three new real-input pins + the gun.rs unit pin, run on this tree: `fix_06_fire_live`
  3 passed / 0 failed; `nf-game` full suite green (unit + live).
- The two walkreal tests were executed un-ignored during the session (then re-ignored, see
  below): with the two route additions the walk runs the truck-ride spawn → the dismount → the
  well lane → the guardhouse fight (guard #2 dies) → door #47 → the guardroom → door #59 → the
  mouth (ch 0x52 on) → down the courtyard → door #71 → the tunnel, climbing to y 46.6.
- The fast runner (`tools/test/suite.py`) on the final tree: see the total line below.

## What's left (honest gaps)

1. **The tunnel ramp's upper half wedges the walk** — Bond pins at (5.45, ~46, −59.04) climbing
   y 42 → 49. Measured fallbacks, all documented in `climb_the_tunnel`'s doc comment: dense
   8-tick jump pulses gained nothing (the ceiling bonk cancels each hop), a crouch-walk slid back
   to 45.45, and a re-center `walk_to` wedges on the ramp surface itself (the eye ray hits the
   slope). The walk is re-ignored with this as the named OPEN item — it is route/geometry, not
   the fire path. Next session: dump the tunnel's floor/ceiling geometry around z −59..−63
   (the port's collision vs [D] disc) — if the disc's tunnel is a walkable slope, the port's
   step-up/floor code is the suspect; if it is stairs, the climb cadence is.
2. **An unwrapped bond.yaw (+77.8 rad, ~12.4 turns) was observed after the mouth walk** (the boot
   NIS region). The fight still aims and fires correctly (the steering wraps), but the port never
   normalises `bond.yaw`; whether the game's own yaw accumulates the same way during the skipped
   boot NIS is unmeasured. Cosmetic today; worth a look when the NIS camera work is revisited.
3. The two 06 walks remain `#[ignore]` until the tunnel leg walks.

## Live checks for the main agent

1. **The tunnel ramp** — render the observation point `--bond 4.4,46.6,-59.0` facing up-tunnel
   toward z −60..−65: what the real game shows there (a walkable slope, discrete stairs, or a
   blocked/collapsed section) decides whether the port's collision or the route is wrong. A
   FRESH-boot walk up the tunnel with the debug HUD (bond y each second) would pin the disc's own
   climb rate.
2. **The boot NIS body spin** — with `gdbwatch.py` on the player object's yaw word during the
   skipped boot NIS at the searchlight courtyard (fresh boot, walk the route to the mouth, skip
   the NIS): record whether the game's yaw accumulates large unwrapped values (the port's
   +77.8 rad) or is re-normalised by `Player_Enable`/the hand-back at free.

## Usage

- Subagent (research, Explore): prompt 278366, completion 11266, total 289632 (from the agent's
  final report footer).
- This session's own token/credit/context figures: not visible to me (no counters are exposed to
  this worker environment; the numbers above are the only ones I could read).

## Suite (final tree, merged with origin/master)

This sandbox kills background jobs between tool calls (two nohup/setsid runs of
`tools/test/suite.py` died silently with empty logs), so the fast runner's single background pass
was not possible; the suite was run in foreground segments on the final tree (all 775 test
binaries built once, then the runner's `--only` chunks + the direct cargo runs):

- **nf-game** (cargo, all binaries, `--test-threads=1`): every binary green (includes the new
  `held_automatic_fires_at_the_record_rate`).
- **nf-audio** (cargo, all binaries): green.
- **nf-formats** (cargo, all binaries): 11 passed, 3 failed — exactly the 3 `disc_golden` tests
  (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`), the task's allowed slim-ISO failures.
- **nf-view slice 1** (the runner, the whole weapon/fire/06 family — 32 binaries):
  `124 passed, 0 failed, 4 ignored (2 min)` — fire modes, rapid fire, the new
  `fix_06_fire_live` pins, the 06 alarm/driver/fusebox/searchlight/interact/walk tests, stunner,
  reload-hang, grenade throw/release, weapon anims, the mp auto volley, the decryptor hold,
  weapon effects, guard first shot, samurai, scorpion, delta repeater, q-claw, bullet range,
  handcannon, weapon modes, weapons-all, gadget laser, melee, laser watch.
- **nf-view slice 2** (the runner, the models-golden + walker-regression family — 16 binaries):
  `56 passed, 0 failed, 13 ignored (4 min)` — `models_world_live`'s goldens unchanged, weapon
  models/misc/sound/census, the level sweep, the walker-heavy walks (01/04/05/0a) and the truck
  fix — no regression from the route additions.

Full 775-binary total line: not obtained this session (the background runner is not survivable
here and the foreground window did not fit the remaining ~600 binaries; the two slices above are
the honest coverage, biased to everything the change touches).
