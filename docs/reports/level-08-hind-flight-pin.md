# level-08-hind-flight-pin

The 07000008 helicopter's flight, measured on the real path and pinned. The task's premise — the
machine "must fly its attack, not hold one spot" — resolved into two findings: the flight itself was
already stall-free (Copter_Move's spline pose, the merged copter-path-flight work), and the ATTACK
(`Copter_Update` 0x17dbb8's case 2) was unreachable — the port's own copter.rs header documented the
spot check as "NOT ported ... the port's copters stay in state 1 and never draw RNG". This task ported
the spot check and the attack, measured the flight on the real route, and pinned it.

## What the game does [M] (all addresses are ACTION.ELF decompile files)

- `Copter_Update` 0x17dbb8, case 1 (fly): `Copter_Move` 0x17d628, then the spot check — behind the
  `+0x74 == 100` Mayhew latch [M :140], a `+0xf6` timer counts up; from `240/DiffMod` ticks [M :142,
  `FRAME_RATE_INT << 2 / DiffMod`] the test runs EVERY tick: the player within 1000 of obj+0xC0
  [M :151], `Intersect_ConeSphere(COPT_COS, COPT_SIN, obj+0xC0, Mat_GetDir(obj+0x90), player+0x80)`
  [M :153-155] — the 45° cone (`COPT_COS = PS2Sinf(0x4016cbe4)`, `COPT_SIN = PS2Sinf(0x3f490fdb)`
  [M 0x17e468:7-8], both ≈ 0.7071), the player's world sphere (radius 1.0576 [L obj-tag-player.md
  +0x80]) — then `Collide_LineOfSight` from `obj+0xC0 + dir·4.0` with flags 0x2e [M :156-172]. A hit:
  timer 0, state 2, rec+0x6c = 0, `attackmode_159 = Rand_Rand(10)` [M :181-184].
- Case 2 (attack) [M :188-243]: `Music_Event(2, 1)` [M :189], `Copter_Move`, the timer, then the
  pick: `attackmode_159 & 1` — 0 → `Copter_FireMissile` 0x17d9f0 (the pods rec+0x4c/+0x50 = part
  slots 7/8; the unconditional `Rand_Rand(0x86e6)` parity draw [M :47]; the `GameState+0x52 %
  60·DiffMod` cadence [M :58-64]; on 07000008 the `Rand_Rand((u32)(8.0/DiffMod)) == 0` homing-AIM
  roll [M :65-76] — the aim turns at the player, the record itself (0x62: +0x68 flags 0 [L
  ee_snapshot.bin @0x2bf150 + 0x62·0x10c]) flies straight; `Bullet_init(−1, copter, pod,
  weapon_data+0x62·0x10c, pod+0x80 + dir·glist_radius·1.5, dir, 0xffff)` [M :79-97]), 1 →
  `Copter_FireGun` 0x17d828 (gun A rec+0x44 = slot 5; every 6th call [M :63-65]; the muzzle
  `gun+0x30 + dir·(*(glist)+0x38)` [M :66-84]; `Sound_Play3D(100, −1, −1, 0x5df, muzzle)` [M :82];
  the weapon id 0x61 on 07000008, 0x71 elsewhere [M :86-89]; `Bullet_init` likewise). The bail rules
  [M :199-242]: the 300·DiffMod window, the 100 attack-damage bail (rec+0x6c), and the visibility
  re-check (the same cone+LOS) — a bail resets the timer, drops to state 1, zeroes rec+0x6c; visible
  stays attacking.
- `DiffMod` = the difficulty multiplier set by `Copter_Create` 0x17d340:74-93 (2→1.5, 3→2.0, 4→4.0,
  else 1.0) — the same value the hp takes.
- `attackmode_159` is ONE global word shared by every copter [M :184].

## The port's fix (crates/nf-game/src/copter.rs, drone.rs, nis.rs, sensor.rs)

- `Copter::cone_sphere` — `Intersect_ConeSphere` 0x1ee100's exact float flow (the apex-inside arm,
  the axial band, the tangent test), with `copt_cos/copt_sin` = the port's `trig::sinf` (the
  PS2Sinf mirror) at the game's two arguments.
- `Copter::spotted` — the case-1/2 spot test verbatim (the 1000 gate, the cone, the 4.0 probe
  stand-off, `world.line_of_sight(probe, psph, 0x2e)`).
- Case 1 runs the spot check behind the `arrival != 100` latch (the 05 patrols carry 100 [L the
  level's params] and keep flying silently, as before). Case 2 implements the attack with the bail
  rules, returning the calls the game makes inline as `CopterEvent::Music / Gun / Missile`; the
  caller (`Drones::tick_inner`) plays `music.event(2, 1)`, the gun's 0x5df at the muzzle, and spawns
  the round through `spawn_copter_bullet` — the `Bullet_init` port the GT turret's Fire arm already
  established (the spread from the record's +0x24/+0x68, `init_fuse`, no extra pad draw — neither
  fire function has one [M 0x17d828/0x17d9f0 tails]). The owner is the port's not-a-drone mark —
  every hit-time shooter read skips, the same NULL-gating the game's copter owner takes.
- `attackmode_159` = the new `Drones::copter_attackmode` field (one word, shared).
- `nis.rs::copter_part_matrices` — the pods' `Mat_GetDir(pod+0x90)` facing (the entity's own matrix,
  or the stream matrix that owns an anim — the same resolution `copter_part_targets` uses).
- `sensor::TARGET_RADIUS` (1.0576) became `pub(crate)` — the same sphere constant the spot cone now
  reads.
- The copter unit tests moved to the new `update` signature through a `upd` shim (a None player —
  the game's `glb_players == 0` arm — and an empty world): 658/658 lib tests green.

## The measurement (walkreal_07000008_live, the real route, no pokes)

The walk's copter section now logs the machine every 30 ticks from the gate (ch 5, the bottom
cinematic) to the death (ch 0x4B) — position, state, path index, t, damage — in all four difficulty
sister runs the binary carries:

| run | hp | samples | travel | states seen |
|---|---|---|---|---|
| walk (Operative) | 299 | 29 | 123.6 u | all 1 |
| real_damage | 299 | 29 | 125.9 u | all 1 |
| real_damage_00agent | 598 | 45 | 148.3 u | 1s + **one 2 at t307.5** |
| real_damage_agent | 448.5 | 43 | 168.2 u | all 1 |

- **No stall**: not one 30-tick flying stretch moves below 2 u (the measured smallest was ~4 u; the
  path's t step is 0.2533/tick [D #45's framelist, the 0x2F record]). The sweep runs
  (70.95, 22.16, −165.04) f25860 → (171.16, 16.32, −192.44) f26640 and back on its re-picks
  (`group_paths` = 1, the own path is the group's only kind-0 member [D the route dump], so each lap
  re-picks the same path at its reset-t).
- **The attack fires on the real path**: the 00 Agent timeline's spot check lands (the cone points at
  Bond during the later circuit — state 2 sampled at f28650, the copter's own round alive in the air),
  then the bail rules return it to 1. The Operative/Agent timelines' circuits never point the cone at
  Bond's fighting spot — the same data-driven answer the game's own cone computes from the same
  [D] path keys and facings (the port's spline pose interpolates the keys' quaternions, the same
  input `Spline_Interp` mode 2 reads).
- **The case order**: 0 → 1 (with 2 wherever the cone lands) → the rockets cross hp → 4 (pinned from
  the sim at the ch-0x4B break; no kind-2 arrival path exists on 08 [D the route dump], so rec+0x58
  stays None) → 5, whose ticks re-write ch 0x4B → the objective folds → the level ends into the
  DRIVING hand-off 0x0730008. All pinned in `walkreal_07000008_live::walk_the_whole_mission_*` —
  6/6 green (travel ≥ 100 u, the ≥ 2 u-per-30-ticks no-stall law, cases 1/2 in flight, the death
  state 4, ch 0x4B only from the death, rockets ≤ 8, the blast's ≈100 per body hit).

## Verification

- nf-game lib: 658 passed, 0 failed.
- copter_paths_live 4p, copter_05_flight_live 1p, copter_death_debris_live 2p,
  copter_script_draw_live 1p, copter_sound_12_live 3p, copter_hits_12_live 3p — all green (the 05
  patrols' and the 12 fight's pinned behavior unchanged).
- gondola_windows_live 3p, zoe_gondola_realpath_live 1p — green.
- walkreal_07000008_live: 6 passed, 0 failed (144 s), with the flight pins inside.
- The walkreal sweep and the full suite: see the numbers below (the session's tail).

SWEEP_RESULTS_PLACEHOLDER

## What's left

- The barrel-aim presentation: the game's `Mat_LookAt` turns the two gun-barrel objects at Bond each
  attack tick [M 0x17d828:60-62]; the port's barrels are script entities posed by their keyed
  streams, and no object matrix write path exists for them — the SHOT (muzzle, sound, round) is the
  visible part here. A future script-entity override hook could carry the aim.
- `LOS_MASKS_ON` is `false` port-wide (the parked flip, world.rs:50): the port's spot LOS is the
  plain segment test, STRICTER than the game's 0x2e drops (which pass glass). The cone gate is
  LOS-independent, so the measured case-1/case-2 split stands; flipping the global would only ever
  let the port see MORE.
- The two difficulty variants whose circuits never point the cone at Bond: the live check below
  asks the PCSX2 run to confirm the game's Hind also holds its fire on those timelines.

## Live checks for the main agent

1. The bottom-terminal fight, 00 Agent: does the real Hind open fire (gun tracers or a missile) at
   any point of its circuit? Expected from the port: yes on the 00-Agent-style approach (the cone
   lands late in the circuit), and the music board takes Music_Event(2,1) during the attack.
2. The Operative/Agent approaches: the port's Hind never enters case 2 from the terminal fight (the
   path keys' facings never point within 45° of the terminal). A PCSX2 watch on `attackmode_159`
   (0x2a37e4-ish .sbss — the exact address needs the symbol table) during those fights would settle
   whether the game's cone answers the same.

## Usage

Token figures visible to me: the Task-tool subagent results this session reported `prompt/completion`
pairs (probe agents 5262/5, 5765/184); my own session's context-window, token totals and any
credit/cost figures: not visible to me.
