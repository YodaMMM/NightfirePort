# level-1b-space-laser — SpaceLaser_Update 0x1aa3f8 wired; the 0700001b laser arms, charges, fires, dies and writes ch 99; the live chain 4/4

## What's done

1. **The sim wiring landed** (crates/nf-game/src/drone.rs, the Drones control pass, next to
   the SpaceMissiles' precedent): every tick each kind-0x38 laser runs
   `SpaceLaser::tick` — control_funcs row 0x50 = `SpaceLaser_Update` 0x1aa3f8
   [M docs/atlas/object.md:164; the create walk's case 0x38 @0x1d06fc,
   parsemap_create_dynamic_objects decomp :225-227]. The tick's outputs are wired to the
   real systems:
   - **The state-2 shot**: the ray from the fire origin along the unit aim dir, 1000 units
     (0x447a0000 [M :135-143]), the game's mask-8 semantics (the pass-flagged hits drop
     [M Collide_Intersect disasm 0x1ebff0-0x1ec010]) = `world.segment_bullet` [M world.rs];
     a hit rewrites the aim lane to the impact [M :153-160]. Then `Bullet_init(-1, obj, 0,
     0x2c6154, origin, dir, 0xffff)` [M :172-175] — the weapon def @0x2c6154 = weapon_data
     0x2bf150 + 107·0x10c = record 107, whose speed/range/damage bytes fill the port's
     Bullet (the same struct the drone fire path uses, so the damage reaches Bond through
     the proven bullet/damage machinery). The s16 −1 owner maps to the port's usize::MAX
     (nothing reads Bullet.owner for credit).
   - **The beam object** (class 0x10, `Debris_Update` 0x1aac00 [M :159-167 + object.md:110]):
     exposed per tick as `Drones::space_laser_beams` (`LaserBeam { from, to, alpha, life }`)
     — the endpoints the renderer draws, life 5 ticks with the Debris_Update decay
     [M 001aac00:20-28], alpha = 0x7f+Rand(0x80) [M :166-167]. The draw is done separately.
   - **The 0x2d6210 impact emitter** [M :189] through `fx::spawn_impact` (the game's
     Emitter_Create's own def-hash-miss = no-object rule [M 0x17ae00]).
   - **The sounds**: the 0x5de charge whine via `Sounds::play` with the handle stored at
     ext lane 3+0x0C and `Sound_Stop`ed on the shared tail [M :119-121, :184-187]; the 0x46a
     3D fire sound [M :156].
   - **The ch-99 write** (state 3, data+0x40 = param 5) and the **ch-14 kill** (data+0x42 =
     param 6 → state 4 → obj+0xfe |= 1 [M :86-90, :189-195]) were already in the unit port;
     the wiring drops the deleted laser the same frame (the game's delete pass).
2. **The script clock**: the game's `Script_Load`+`Script_Play`+per-tick `Script_Update`
   lifecycle runs as a tick counter against the **[D] lib lengths** read from the level's
   NIS library at the load (the tracking pair 0x060007c0/c1 = 360, the dead script
   0x060007e3 = 10 [D docs/levels/0700001b-audit.md:606-612]); the load tick's own
   Script_Update counts [M :103/:112]; the clock advances in states 1 and 3 only
   [M :116/:176]. `Script_IsPlaying`'s End-marker exit [M Script_StreamEnd 0x1db420] is the
   length boundary.
3. **Two creator-seed fixes**: `from_placement` now seeds the cooldown with the creator's
   constant 10.0 [M 001aa8b8:38 — ext+0x38 = 0x41200000 at create]; the old 0.0 seed fired
   on the first gate tick. The unit test that pinned the 0.0 behavior was WRONG against the
   decompile — fixed with the address (the_armed_fire_path now drives the 600-tick seed
   window first). The fire origin seeds from the placement's own position — the level's
   kind-0x38 placement IS the "BigLaserBeam Fire Marker" at (0,0,0) [D
   0700001b-audit.md:275], and the game's `SpaceLaser_Register` 0x1aa9e8 fills lane 0 from
   the tracking scripts' registered entities, which `Script_CreateEntity` 0x1d8968
   bare-creates at the zero quad [M 0x1d8968 fallthrough + 0x1d8d78's
   `Vec_Copy2(+0xc0,+0x40,+0x30)` of zeros; the stream decode shows no position ops,
   D audit:611-612].
4. **BigLaserBeamHURT (0xe4 #259, type 1, dmg 50, gate ch 18 [D audit:444-445])] arms
   through its gate: `Hurt_Update` starts a gated volume on the live channel
   [M 0x17e5d0:26-37, :51-102] — pinned live on 0700001b (parked while ch 18 is low, armed
   once it rises).

## How it was verified

- **crates/nf-view/tests/space_laser_1b_live.rs — 4/4 GREEN** through the real Sim (the
  Walker harness, the level's start switch included):
  - the_laser_arms_charges_and_fires_at_bond: gate held → the tracking script loads with
    its [D] 360-tick length → state 1 with the 0x5de whine handle → the fire: the beam
    (life 5, alpha ≥ 0x7f, to = the aim lane), the record-107 bullet when it survives the
    tick (the pit-centre muzzle can die on the geometry the same tick — the beam and the
    emitter are the durable evidence), the 10.0 cooldown reset, and the 10 s re-fire.
  - ch_14_kills_the_laser: ch 14 → state 4 → deleted → gone the same frame.
  - ch_99_rises_in_state_3: the dead script's [D] 10-tick load → state 3 → the [D] 65.0 s
    re-raise → ch 99 rises and holds. The 165 s arm-delay wait is poked to just under 0
    (the suite cannot afford 9900 ticks; the unit test pins the real decrement rule) — and
    the missiles are inerted (state 4) because they share the ch-19 gate and their own fail
    chain writes ch 99 @t1000 [D space_missile.rs's stream decode], which would mask the
    laser's write.
  - big_laser_beam_hurt_arms_through_its_gate: as above.
- **nf-game lib: 578 passed / 0 failed** (the space_laser unit family: the param map, the
  inert gate, the kill gate, the armed fire path on the corrected seed, the state-3
  re-raise).
- **The full four-crate suite did not run this window** (the background fast runner dies at
  the tool-call boundary in this sandbox family — measured again; the foreground budget went
  to the wiring and the live chain). The touched crates' tests ran: nf-game lib (578/0) and
  the new nf-view live file (4/4). The next window runs tools/test/suite.py per the standing
  rule.
- Harness notes pinned in the test doc: the level's start clears switch_channels around
  frame ~49 (a one-shot ch-19 poke survives exactly one tick — the gate is re-poked per
  tick; the live check's debugger poke fights the same clear).

## What's left

1. **The full suite** on the final tree (fast runner; the totals line).
2. **The renderer's beam draw** — the endpoints/alpha are exposed per tick
   (Drones::space_laser_beams); the draw side is a separate task (the merge render should
   show the beam down the pit — see the live checks).
3. **The emitter def 0x2d6210's global resolution** — the port's fx.defs carry the level's
   0xF2 placement defs; whether the game's global effect table holds 0x2d6210 (the game's
   Emitter_Create would then spawn the impact) needs the live check below.
4. **The tracking scripts' register semantics** — the decode shows bare entity creations
   (no position ops); a live measurement of the real muzzle settles the (0,0,0) seed.

## Live checks for the main agent

1. **The muzzle/beam origin**: fresh boot 0700001b, poke ch 19 (the debugger), wait ~11 s —
   where does the beam start and end? Expected per the port: from (0,0,0) (the Fire Marker
   placement [D audit:275]) toward Bond's position, stopping at the first non-pass surface
   (or 1000 units out). A different origin means the tracking scripts' registered entities
   carry positions the stream decode missed.
2. **The 0x2d6210 emitter**: the same boot, at the fire tick — does an impact emitter spawn
   at the beam's end (the game's global effect table hit)? The port spawns it only when the
   def resolves.
3. **The ch-18 writer**: BigLaserBeamHURT's gate — watch who writes ch 18 (0x12) during the
   laser's state-3 window (the port's script streams show no channel writes [D audit:606];
   the mission-fail chain is the suspect).

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).
