# SpaceLaser_Update 0x1aa3f8 — the arm/fire state machine (port plan)

Researched and documented 2026-10-21; the code port is
**remaining work**. The creator's data is already ported (`crates/nf-game/src/space_laser.rs`,
`from_placement` = `Create_SpaceLaser` 0x1aa8b8). The disc ships exactly one kind-0x38
placement: 0700001b PL#108 — armed by switch channel 19, killed by channel 14 [D
script-census.md:144; docs/levels/0700001b.md:71].

## State block

`obj+0xE0` = the ext pointer `pauVar3` of 16-byte VU quads [M 001aa3f8:59]:

- `pauVar3[0]` = the laser's position (obj+0xC0 quad; the `SP_SetPosRot`-style script
  callback reads obj+0xC0..0xD0 [M 001aa9e8:12-20]);
- `pauVar3[1]` = the aim/target quad (written from `glb_players+0x30..0x40` — player 0's
  position — when arming [M :104-110], and from the ray hit at +0x40..0x50 when firing
  [M :147-153]);
- `pauVar3[2]` = {+8: one of two tracking script ids, +0xc: the loaded SCRIPTINFO*}
  [M :89-101];
- `pauVar3[3]` = the float clock block {+0: initial arm delay, +4: post-kill channel re-arm
  timer, +8: fire cooldown, +0xc: sound handle} [M :61,:82-87,:119-120,:172,:177-181];
- `pauVar3[4]` = channel block {+0: kill channel (param 6 = 14 on the disc), +2: kill-gate
  channel (param 5's pair, 0 on the disc), +4: arm channel (param 7? the disc value 19)}
  [M :60,:71,:179-181].

`obj+0xF4` (u16) = the state, 0..4 [M :62-77]. The port's `SpaceLaser` struct already
carries the placement params (`ch5`/`ch6`/`ch7` = params 5/6/7's low u16s) and `state`.

## The machine (all [M 001aa3f8], line numbers from the decompile file)

Pre-switch (every tick, :60-66): if the arm channel word (+4) is 0 **or**
`switch_channels[ch] != 0`, decrement `pauVar3[3]+0` (the arm-delay float) by
`REC_FRAME_RATE`.

- **Kill gate** (:71-74): if `pauVar3[4]+2 != 0` and `switch_channels[+2] != 0` →
  `obj+0xF4 = 4` (killed) regardless of state.
- **State 0 (idle)** (:78-114): if the arm channel is open: stop the sound handle
  (+0xc, :81-84); copy cooldown `pauVar3[3]+8` → local and decrement it (:85-87);
  - arm delay < 0 (:88-95): `Script_Load(scripts[1], 0x2d6200, 0x2d6200, 0,0,0,0)` (no
    callback), `Script_Play(.., 1)`, `Script_Update`, → state 3 (the killed/idle script).
  - else cooldown < 0 (:96-112): `Rand_Random() & 1` picks scripts[0..1]
    (:97-99); `Script_Load(..., 0x1aa9e8, pauVar3)` — **the Register callback supplies the
    laser's position to the script**; `Script_Play(.., 0)`; `Script_Update`; copy
    `glb_players+0x30..0x40` (player 0 pos) into `pauVar3[1]` (:104-110); → state 1.
- **State 1 (tracking script playing)** (:115-127): `Script_Update`; when
  `(u16)(SCRIPTINFO+0xab0) >> 1 < (float)(SCRIPTINFO+0xa0)` and no sound handle:
  `Sound_Play(100.0f /*0x42c80000*/, 0x5de, 0)` — the laser whine, sound id 0x5de
  (:117-120); when `Script_IsPlaying(..) == 0` → state 2, clear the SCRIPTINFO handle
  (:122-126).
- **State 2 (fire)** (:128-174): aim = normalise(`pauVar3[1] − pauVar3[0]`) (:129-133);
  ray from `pauVar3[0]` along `aim · 1000.0f` (0x447a0000) via `Collide_RayIntersect`
  (cel obj+0x20, mask 8) (:134-146); on hit, copy the first hitdata's +0x40..0x50 point
  into `pauVar3[1]` (:147-154); `Collide_FreeHitList` (:155); `Sound_Play3D(100.0f, −1.0f,
  −1.0f, 0x46a, pos, 0,0,0)` — the shot (:156); `Bullet_init(−1, obj, 0, 0x2c6154,
  pos, aim, 0xffff)` (:157-158); `control_create_object(0x40, pos, 0, 0)` — the beam
  object, class 0x40 (:159) — then `ext+0xff = 0x10`, beam ext+0x30 = 5, ext+0x32 = 0
  (:162-164), `hashtable_set_object_to_entity_gfx(beam, 0x20005cd)` (:165),
  `ext+0x106 = Rand_Rand(0x80) + 0x7f` (:166-167), link to
  `build_FindCel(pos, glb_world)` (:168-169), `PositionBeam(beam, pos, pauVar3+1)` — the
  beam runs laser→hit point (:170); `Emitter_Create(0,0, 0xc000010, pauVar3+1 /*hit
  point*/, 0x2d6210, 0,0,0)` (:171); reset the cooldown `pauVar3[3]+8 = 10.0f`
  (0x41200000) (:172); → state 0 (:173), then the shared tail (below).
- **State 3 (killed script / wind-down)** (:175-188): `Script_Update`; decrement
  `pauVar3[3]+4`; when < 0 and the kill channel word `pauVar3[4]+0 != 0`:
  `switch_channels[kill] = 1` and `switch_channels_time[kill] = GameState+0x52`
  (:179-182) — **the laser re-raises the kill channel (14) itself**, which is what kills
  Drake's trigger; then the shared tail.
- **State 4 (dead)** (:189+): stop the sound handle if set; stays.
- **Shared tail** (:183-188): if the sound handle != 0 → `Sound_Stop` and clear it.

`SpaceLaser_Register` 0x1aa9e8 (24 bytes, the `Script_Load` callback): copies
`obj+0xC0..0xD0` (the position quad) into the out pointer when non-null [M 001aa9e8:12-20].

## Port plan (next session)

1. Extend `SpaceLaser` (`crates/nf-game/src/space_laser.rs`, fields on their own lines):
   `target: [f32; 3]` (pauVar3[1]), `script: Option<script::Script>` (the port's NIS
   `Script` stand-in, `nis.rs`), `clocks: [f32; 3]` (arm delay, re-arm, cooldown),
   `sound: Option<SoundHandle>` (the port's sound-coverage model; class 0x50 is a
   documented sound gap — sound-coverage.md:95).
2. `SpaceLaser::update(&mut self, ...)` mirroring the five states with the port's switch
   channel API (`switch_channels` equivalent), the port's ray (`Collide_RayIntersect`
   stand-in), `Bullet_init` and a beam-object record (the game's class-0x40 object: store
   `{pos, hit_point}` — the draw side is separate).
3. `register_pos(&self) -> [f32; 3]` = the Register callback (returns obj+0xC0).
4. Real-path test: 0700001b live (the placement's params drive ch 5/6/7), assert: idle →
   (arm channel 19 set) state 1 with player-0 target; state 2 fires a beam + resets
   cooldown 10.0; kill channel 14 → state 4; state 3 re-raises the kill channel after its
   timer [M :179-182]. Sister assertions: without the arm channel nothing fires; the
   damage path never runs while the kill gate (+2) is 0.
5. Sound: id 0x5de (whine) and 0x46a (shot) are MISSING coverage — list in
   sound-coverage.md when porting.
