# level-1b-platform-touch — the Drake-fight platforms' break rule

Task: on 0700001b the platforms in the Drake fight "break at the slightest touch". Find the
game's break rule for them (damage source, threshold, what counts as a hit) in the decompile,
compare it to the port, fix the difference, and prove it with a walk-on test (no break) and a
break test. Branch `s/level-1b-platform-touch`.

## What the platforms are

The 16 "platforms" are the solar-panel placements #41-56 around the Drake arena (flags 0xd9,
param0 script 0x06000832, param7 0x14 = 20 hit points, z=±4.62, x 10.14..56.12) [D
0700001b.bin placements; probes `crates/nf-view/tests/probe1b_platforms*.rs`]. The panel
script 0x06000832 spawns `OBJECT_SolarPanelWhole` 02000a42 (the intact panel) and holds;
10 fragment streams (`OBJECT_SolarPanel1..10`) and an Event kind 3 play when the hold
releases — the break choreography [D script decode, probe 3].

## The game's break rule (decompile)

- **Damage source — bullet contacts only.** The Collide pass fills each object's `+0xD0`
  HITDATA chain fresh every frame [M `Collide_Update` 0x1ea398's first loop frees, second
  loop re-picks]. `SP_Update`'s damage tail runs only while the SP's current script is HELD
  (`SCRIPTINFO+0x9c != 0`) and the SP has hit points (`SP+0x14 != 0`), sums the chains of the
  CURRENT script's stream objects via `SP_GetHitDamage` 0x1dc5f8, accumulates into `SP+0x18`,
  and at `SP+0x14 <= SP+0x18` calls `SP_UnPause` and subtracts the threshold (the remainder
  is kept) [M SP_Update 0x1dc1c0, decomp file 001dbda8:236-267]. `SP_GetHitDamage` filters
  with mask 0, which drops nothing [M `Collide_FilterBullets` 0x1ece30:24-31 — only a
  non-zero weapon-category match stamps a record], and `Collide_GetDamageNObjects` 0x1ee220
  sums EVERY record's +8 damage — bullet records are class 5; there is no other damage class
  feeding the chain. `Destroy_CollisionHandler` 0x12bd48 (the 0xE1 destroyables) likewise
  subtracts only class-0x05 records [M] — **no player-contact break path exists in the
  engine**.
- **Threshold — placement param 7.** `SP_CreateScriptPlayer` 0x1db9a8 stores param 7 into
  SP+0x14 [M; the port's `Player::from_placement`, nis.rs]. The panels carry 20 [D]. The
  start weapon 0x33's damage is 20 [D weapon_data], so one 20-damage round breaks a panel BY
  DESIGN.
- **What counts as a hit — the glist bounding-sphere test.** `Collide_PickObj` 0x1eae80
  gates the pick on obj+0xF0 bits vs the hittest word, on the object class for hittest-word
  bits 0x400/0x100/0x200 (classes 3/0x12, 2/0x11, 5), and requires `hittest word == 0x100`
  or `obj+0xfa & 8` or a glist record with data at +8; then the sphere test on
  obj+0x80/obj+0x8C — `Control_BuildWorldSph` 0x133070 builds them from the glist record's
  stored centre (+0x10) and radius (+0x1c) × scale [M]. `Bullet_update` 0x12e7b0:432-458
  arms the pick word +0x94 = **0x201** (fast bullets: the closest-point-on-segment arm with
  the object's own radius) or **0x101** (slow rounds: a point/radius-sum arm) [M]. The panel
  entities are plain `control_create_object` objects (type 0x31 is not one of
  `Script_CreateEntity`'s special types [M 001d8968]) with collision data in their glists
  (SolarPanelWhole: 1 box, 24 tris [D probe 4]) — they pass the gates and are pickable.

## The port's side — comparison

The port implements the same rule on every point checked: the damage tail (hold gate, hp !=
0, accumulate `tick_damage`, unhold + subtract at the threshold) in `nis.rs`
`Player::update`; the mask-0 sum over the held current script's stream objects
(`Scripts::held_hit_targets`); the bullet-pass contact test `segment_hits_sphere(b.pos, end,
centre, radius)` with the model glist sphere — the game's 0x201 arm [M 0x1eae80]; the glist
sphere from the entity-params centre/radius (nf-formats level.rs, the same stored values the
game's BuildWorldSph reads). `Scripts::add_hit_damage` is called only from the player-bullet
pass (drone.rs) — no touch/blast path feeds panel damage. The SP_Hit touch bits (8, 0x10)
are unmodelled, which cannot break panels either (the panels' trigger mask is 0 [D
placement param 5], so no SP_Hit bit triggers them).

**Difference found: none in the rule itself.** The port breaks a panel exactly where the
game's rule does: a bullet segment touching the panel's glist pick sphere, 20 damage at the
threshold, hold released. The reported "breaks at the slightest touch" did not reproduce
through the real input path: see the tests below and the live checks requested.

## Proof (real input path from the level start, no teleport)

`crates/nf-view/tests/platform_touch_1b_live.rs` (both green, NF_ISO, --test-threads=1):

1. `walking_on_a_platform_breaks_nothing` — after the arrival scene frees Bond (3000 ticks),
   the test walks him onto the panel walkway (placement #50, the walker's steering incl.
   zero-g) and stands/steps there for 900 ticks: all 16 panels stay held, zero accumulated
   damage, no debris fragments. Matches the engine: no touch break path [M 0x12bd48].
2. `a_bullet_contact_breaks_a_panel` — the same walk, then switch to the start weapon 0x33
   and fire: a panel unholds (its `delay` releases) and the debris fragments spawn over the
   following 300 ticks. The game's break action is a bullet contact at the 20-damage
   threshold [D weapon 0x33 damage 20; M SP_Update 0x1dc1c0:265-267].

Investigation probes (assertion-free, committed): `probe1b_platforms{,2,3,4}.rs` (the
placements, the script decode, the glist collision census) and `probe1b_touch_repro.rs`
(the walk/fire session this analysis read its numbers from).

## What's left / open questions

- The user-visible symptom ("breaks at the slightest touch") is NOT reproduced in the port
  sim. The one geometric effect that can read as "slightest touch": the panel pick spheres
  (radius ≈ 4.77 [D glist]) overlap along the walkway, so a bullet fired anywhere on the
  walkway starts inside one or more panel spheres and breaks the nearest panel on its first
  flying tick — the game's identical sphere test [M 0x1eae80/0x12e7b0] does the same, so
  this is game behaviour, not a port deviation.
- One port deviation stays as documented [*deviation* in drone.rs]: the game's Collide pass
  covers drone-fired rounds too (Drake's fire breaks panels in the game); the port's
  drone-bullet pass is separate and stays out of the read set — the port is strictly harder
  to break than the game, never easier.

### Live checks for the main agent

1. `gdbwatch` on a panel SP's `SP+0x18` (accumulated damage) and `SP+0x14` (hit points) while
   Bond walks the panel walkway in the real game (0700001b, the Drake fight, stand on the
   walkway between the missile and the arena edge): expected — +0x18 stays 0.000 the whole
   walk (no touch damage [M SP_Update 0x1dc1c0:236-244 only sums the chains]).
2. Same addresses while firing the start weapon down the walkway: expected — +0x18 jumps by
   20.0 on a shot, and at 20 the script unholds (+0x9c → 0) and the panel plays its debris.
3. `fieldprobe` screenshot at `--bond 16.7,-1.0,4.6,yaw toward -x` on the port after the fix
   branch: the walkway panels intact around Bond; and after a fired shot, the nearest
   panel's debris sequence.

## Verification log

- `platform_touch_1b_live` 2/2 green (the proof tests); `probe1b_touch_repro` run
  (assertion-free).
- Full suite, core crates (`nf-game -p nf-formats -p nf-audio`, --test-threads=1): every
  binary green except `disc_golden` 11 passed / 3 failed — the 3 expected slim-ISO failures.
- The `nf-view` half of the suite did not fit this session's window (serial run exceeds it)
  — no `crates/*/src/` code was changed by this task (new test files only, `common/` only
  read via `#[path]`), so the affected-binary sweep is the task's own binaries above; the
  next session should still run the full `nf-view` sweep before merging.

## Usage

not visible to me
