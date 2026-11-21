# s/mp-bot-drops: multiplayer bots drop their gun when they die

This change moves the shared multiplayer random stream: every bot death draws one `Rand_FRand`
(two for the assault rifle), as the game does. Single player is untouched (all new code runs only
in a match; see Proof).

## What a player notices

- In a match, a bot that dies drops the gun it held. It falls from the bot's hands, turns, and
  lasts 30 s; anyone alive can take it (players first, then bots). It carries a random part of a
  clip, like a guard's drop. Online every machine creates the same gun on the same tick.
- Online, a player who died crouched no longer drops the gun at a different height on each
  machine (that split the machines' world state; seen in the 8-human explosives run once the bot
  drops shifted it onto a crouched death).

## The game's rule [M]

- `NDrone2_DSTATE_BotDeathAnim` 0x175508: ENTER (msg 1) barks, scores (`MP_PlayerKilled`), starts
  the death anim; UPDATE (msg 3) calls `DroneWeap_DropWeapon`.
- `NDrone2_DSTATE_BotDeathByExplosion` 0x1755a8: ENTER barks, scores, then calls
  `DroneWeap_DropWeapon` right after `MP_PlayerKilled`.
- Which state: a round's death enters 0xF2 (`NDrone2_DealWithObjHit` 0x13fff0 case 0x27's MP arm,
  `uVar8 = 0xf2`; `NDrone2_PunchImpact` 0x145f28 likewise); a blast death enters 0xF3
  (`NDrone2_ExplosiveImpact` 0x146480's MP arm, `param_3 = 0xf3`). `Drone_SM_SetState` 0x1758c0
  only queues the state.
- `DroneWeap_DropWeapon` 0x17a2a0: held byte `anim+0x62` (written with `Drone+0x768` by
  `BOTWEAP_changeWeapon` 0x12a028:47); returns below 1; with a third-person model
  (`weapon_data[w]+0x80`): the weapon datum's bone matrix, moved to `obj+0x30` when the ray from
  there hits geometry (mask 0x125); n = `weapon_data[w]+0x92`; f = FRand(0.5) + (n < 10 ? 0.5 :
  0.25); `Pickup_CreateSimple(M, weapon_data[w]+2, (short)(int)(n·f), 0)`; base 0x1A adds 0x1B as
  kind 1 with `clip(0x1B)·(FRand(0.5)+0.25)`, hidden (`obj+0xF0 |= 0x10`); then the byte is cleared.
- `Pickup_CreateSimple` in a match: 1800 ticks, spin, no respawn [M 0x18f440-0x18f47c, 0x18dda0:138-144].
- `BOT_respawn` 0x124e38 deletes the object and runs `BOT_init` (a fresh byte).

## Changes

- nf-view `scene.rs`:
  - `Sim::mp_bot_drop` (per slot): set at every bot death site: the bot-vs-bot round
    (`step_mp_bots`), `mp_hurt` (round = death anim, blast = explosion death), the destroyable
    blast in `step_net_combat`, and the match-decided death in `step_mp_scenario_sync`.
  - The death-anim drop runs at the next tick's bot pass (before the brains step); the explosion
    drop runs right after the kill is scored.
  - `Sim::mp_bot_death_drop`: the port of `DroneWeap_DropWeapon` for an MP bot, on the shared
    stream, through the same reserved draw slots as the human drops (`place_mp_drops`, split out
    of `mp_death_drop`).
  - `mp_drop_reservations`: 2 slots per weapon per body, humans and bots (was humans only).
  - The online human drop reads the body with the dead camera's crouch rule applied (crouch drop
    0) on every machine: the local body had it zeroed, the replicas did not.
- Docs: `docs/systems/pickups.md`.

## Tests

- New `mp_bot_drops_live` (5): a shot bot drops on the tick after the kill (base id, rounds in
  the FRand range, 1800 life, spin, falling, reserved drawn slot with vertices at the pickup's
  matrix, near the bot, lands, spins, once per death, taken by the human); the rounds equal the one
  FRand draw from a copy of the stream; a blast death drops at once; an untaken drop expires after
  exactly 1800 updates; online (two sessions, loopback lockstep through `net_tick`) both machines
  create the same drop in the same slot on the same tick and agree on every tick hash to tick 400.
- Fail-first: with the bot drop switched off, all 5 fail; all 5 pass with it.
- All 59 `mp_*` / `net_*` nf-view binaries: 238 passed, 0 failed (the opt-in two-window test and one ignored test skipped). On the first run `mp_stress_explosives_live` desynced at tick 6985 (the crouched-death human drop, fixed above) and `net_resync_live` timed out once on machine load; both green on the re-run.
- Single player: `load_scene_equality`, `walkreal_07000005_live`, `walkreal_07000002_live`: 1 + 5 + 16 passed (1 ignored, as on master); no single-player pin was touched.

## Digest changes

- `mp_perf_16_live` (2 humans + 14 bots, 3600 ticks): 0x25c0_6d1e_41bf_5788 (36 kills) ->
  0xb6fb_5b05_bf9b_0ccc (43 kills). The bots' drops draw the shared stream and add takeable guns,
  and the bot reservations add pickup entries the world hash folds. With the bot drop and the bot
  reservations switched off on this tree the old value comes back exactly. The machines agree on
  every tick.
- No other pinned digest moved (the humans-only lockstep pins have no bots).

## Unproven / not done

- The drop's start matrix is the bot body's resting gun matrix (the human drop's shared-state
  stand-in), not the posed weapon datum (the port's bot pose is draw-only state). A few
  centimetres to tens of centimetres before the fall; not compared live.
- The death-anim drop tick (one tick after the kill) assumes the bot's UPDATE runs on the next
  tick; whether the game's object order runs it in the same frame as the hit was not checked live.
- A drop is made before the drones' tick, so its first `Pickup_Update` runs on the drop tick
  (lifetime 1799 after it); whether the game updates a new pickup in its creating frame is not
  traced.
- The match-decided deaths (GoldenEye strike etc.) drop through the death-anim timing; the state
  the game's strike victim enters is not traced.
- Weapons outside the reservation list (thrown explosives such as 0x2A, 0x37) still drop undrawn
  (logged), as before for the humans.
- Not compared with the real game.
