# control-funcs-collide — the collide slot of the class control-funcs table

Branch `s/control-funcs-collide` from `origin/master` (10956badb). The table `control_funcs`
@0x26f890 (85 entries × 12 B: update / **collide** / delete) was dumped entry-by-entry from
ACTION.ELF. Twelve classes have a non-NULL collide slot: 0x02, 0x03, 0x05, 0x0f, 0x12, 0x1a,
0x1c, 0x21, 0x28, 0x42, 0x4a, 0x4d [M per-entry dump; cross-checked docs/atlas/object.md:95-166].

**Dispatch fact the whole audit rests on** [M `Collide_Update` 0x1ea398:55-69]: the collide slot
of every object on `DynamicObjList` is called once per frame, unconditionally, whether or not the
object has contacts — the per-frame HITDATA list at obj+0xD0 is optional *input* (rebuilt
0x1ea398:24-158, mirrored to victims :160-233). This is why `ThirdIcon_Update` lives in the
collide slot and works: it runs every frame after movement, after the player's pick mirrored
fresh nodes, and after `Player_CollisionHandler` 0x1949a8:22 cleared BL+0x95F in the same walk
(player is earlier in list order) — clear-then-arm, every frame.

## Per-class audit (game handler → port contact path)

| class | game collide slot | port path | verdict |
|---|---|---|---|
| 0x02 Drone | `Drone_CollisionHandler` 0x140968 | drone.rs `collision_handler` (6083) + bullet/smoke/hurt arms in their owners | PARTIAL — see "left" |
| 0x03 / 0x12 Player | `Player_CollisionHandler` 0x1949a8 | player.rs `physics_tick` + scene.rs icon reset + watch.rs + monitor_air | COMPLETE (dead-state + ceiling tails unmodelled, low impact) |
| 0x05 Bullet | `Bullet_CollisionHandler` 0x12ce80 | bullet.rs + drone.rs player-bullet pass + grenade.rs | COMPLETE for the modelled weapon set |
| 0x0f Explode | `Explode_CollisionHandler` 0x17cde0 (+ `Explode_Propagate` 0x17c518) | explode.rs + drone.rs `spawn_blast` | was PARTIAL — **case 5 fixed this session** (below) |
| 0x1a RB | `RB_CollisionHandler` 0x1d7a80 | rigid_body.rs `rb_collision_handler` 910 (+ arming/consumption in drone.rs) | COMPLETE (bullet+blast impulses, type masks, grounding) |
| 0x1c Door | `Door_ReactToHit` 0x135980 | door.rs `collide_step`/`react_to_hit` | COMPLETE — game takes player+drone contacts only (0x135980:47-72), port matches |
| 0x21 Destroy | `Destroy_CollisionHandler` 0x12bd48 | props.rs `bullet_hit`/`smash`/`destroy_colour_tick`/`channel_kills` | COMPLETE (filter word, health, colour, blast path) |
| 0x28 Car | `Car_CollisionHandler` 0x131838 | car.rs — no contact walk (state machine only, `CarSystem` test-only) | **closed for SP by census** (below); MP-only content |
| 0x42 ThirdIcon | `ThirdIcon_Update` 0x196e80 **in the collide slot** (update slot NULL) | scene.rs icon arm loop (3294-3393) | was PARTIAL — **kill-channel branch fixed this session** |
| 0x4a Sub | `Sub_Collide` 0x1ad868 = 8-byte `jr ra; nop` [M disasm] | none | **closed by census: 0 placements on all 34 archives** |
| 0x4d MiniSub | `MiniSub_Collide` 0x182128 = 8-byte `jr ra; nop` [M disasm] | none | **closed by census: 0 placements on all 34 archives** |

The port-comment sweep ("do not trust a comment that says inert") found **three stale comments**
— all claiming something was not ported when it was; none of the "inert" claims on live paths
was false (the `rigid_body.rs:707-709` grounding-nudge inert claim is [M]-correct: the game's
own `Vec_IsEqual` gate kills it while rb+0xC0 is zero):

- `copter.rs` — "the explosion itself is not ported (no Explode objects in the sim)" → the
  copter's case-6 death blast fires as the `death_blast` event into `pending_blasts` →
  `spawn_blast` (drone.rs:19228-19233). Corrected.
- `explode.rs` — "the debris spawns … are not ported — the port has no debris system" (×2) →
  `Debris_CreateEx` 0x1ac808 is ported in the same file (`create_debris`/`debris_arm`,
  `DebrisSpawn`). Corrected.
- `drone.rs` `spawn_blast` header — "the port has no copter/vehicle/MP/GT/sensor/monitor/sub
  lists yet" → the copter/MP-site/GT arms are in the same function; the rest is closed by
  census/game stubs (sensor `Sensor_ApplyDamage` 0x1b08a8 and monitor `Monitor_ApplyDamage`
  0x1b0968 are 8-byte stubs [M symtab sizes]; sub 0x3d never placeable [D]; vehicle MP-only
  [D]). Corrected.

## Census (the count proof)

`crates/nf-view/tests/collide_column_census.rs` — `Level::parse` over all 34
`nf_view::dev::LEVELS` archives, every placement mapped through the
`parsemap_create_dynamic_objects` 0x1d04f0 dispatch (all 73 arms verified against the jump
table @0x304B20; 0x17/0x19/0x23/0x47/0xdd/0xf6/0xf7/0xff hit the shared default arm 0x1d0f98).
Totals: drone 0x02=588, RB 0x1a=35, door 0x1c=447, destroy 0x21=197, icon 0x42=44, car 0x28=55,
**Sub 0x4a=0, MiniSub 0x4d=0**, player/bullet/explode 0 (runtime classes). Assertions: the two
stub classes never place (closes them with the count as proof); every car placement on a
campaign archive (≤0x0700001b) is at the exact origin with no params (07000002 ×2, 07000025 ×1 —
dummy rows), and every positioned car sits on a non-campaign archive (0x07000023-29, 0x07000046,
0x0700004b — none has a campaign loading image: `ResetMap_LevelCode2Img` 0x1d5258 has no case
for 0x46 and groups the rest with the MP image). `Car_Create` 0x1305b0 refuses only when MP is
on with vehicles off [M 0x1305b0:14-16], so the car object would exist on those maps — MP
content, out of the port's SP scope (docs/systems/mp-never-mentioned.md). The car contact walk
(0x131838's damage accumulate >200/1 → `Car_Deactivate`, the 0.64 Align2Up, the class-0x27/0x28
instant-deactivate) stays a **hand-off stub**; nothing in SP campaign play can reach it.

## Fix 1 — `Explode_Propagate` case 5: a blast cooks off grenade-family bullets

The game's case 5 [M 0x17c518:141-162] takes every reached class-5 victim whose ammo-record id
is in {0x2B, 0x34..0x37, 0x3A} and calls `Bullet_handle_object_destruction` 0x12cd98, whose own
record gate `(rec+0x68 & 0x4000) && (rec+0x70 & 0x80204)` [M 0x12cd98:16-19] is what actually
admits a round: the disc records [L ee_start_05.bin @0x2bf150 + id·0x10C] make the set that
really cooks {0x34 frag, 0x35 stun, 0x36 smoke, 0x37 remote mine, 0x3a tripbomb} — 0x2B passes
the id gate but its record +0x68 = 0x10000 lacks 0x4000; the satchel 0x3b fails both. The
victim is delete-marked (+0xFE |= 1) and `Explode_Create(rec+0x08, rec+0x08, rec+0x0C, bullet,
bullet_pos, up, 0x6000052, 0xff, 0xff, 100, BU+0x30)` fires [M 0x12cd98:20-26] — at FULL record
power (the falloff amount is only the reach gate; the case never reads it).

The port had no case-5 arm in either blast path. Fixed in `spawn_blast` (drone.rs, after the
case-0x4B mine chain) and in `run_beam_explodes` (the mounted laser's own Propagate caller —
the game's one Propagate serves every blast source). Both gates read the live `weapon_data`
records, so the sweep is data-driven exactly like the game. The cooked blast queues into
`pending_blasts` → drains the NEXT tick, the game's order (the new explosion object propagates
on its first collide pass, the frame after — the dispatch walk never revisits the list head it
started past [M 0x1ea398:55-69]). The tripbomb's 0x80000 tail (delete BU+0x38, the beam) is
inherently covered: the port recomputes beams per tick from the stuck grenade, so state=2
removes both [M 0x12cd98:28-34].

**Red test** `crates/nf-game/tests/blast_cooks_off_grenades_live.rs` on the real 07000005 from
the level start (the [L ai_live.rs] start spot; no teleports — the grenades fall and fly through
the real world): run 1 measures where the primer frag settles+explodes (deterministic sim); run
2 (fresh, same load) drops a stuck remote mine at that measured spot. Old code: **FAILED** —
"no cook-off blast at t=211" (the stuck 0x37 has no fuse death, so no blast ever fires from its
position). Fixed code: primer blast at t=210 (0x34, 8.0/100.0), mine cook-off at t=211 (0x37,
8.0/100.0 = full record power) from the mine's position, mine removed. Sister test: the mine 10
units out (outside the 8-unit blast sphere) stays live.

## Fix 2 — the ThirdIcon kill channel (class 0x42's collide-slot branches)

`ThirdIcon_Update` 0x196e80's branches: cooldown tick (:15-17), watch-channel arm (:18-20),
**kill-channel self-delete (:21-27)**, the armed+contacts gate (:28-30), the player walk with
the BL+0x95F write and the type-6 prompt (:31-63). The port modelled all but the kill channel.
It is reachable: the disc's 44 icons include exactly one with kill ≠ 0 — 07000041 map 99's
ctx-2 icon at (8.59, 0, −29.47), kill channel 0x6e=110 [D the 0xFA census], and map 99 carries
the real writer: a TouchOnce trigger (0xEA) at (15.60, 0, −33.22) with out channel 110 [D the
placement rows]. Fixed: `Icon.kill_channel` (wire.rs, populated from placement param 2) and the
arm loop drops a killed icon (scene.rs) — the same observable as the game's delete (nothing
else reads the resolved entry).

**Red test** `crates/nf-view/tests/third_icon_kill_live.rs` on the real 07000041: Bond at the
icon, on-channel armed → `icon_ctx == 2`; the kill channel 0x6e written through the port's
`Channels` (the exact `switch_channels` write the trigger's fire lands in [M 0x1b1a78's Out
copy → `Channels::set`]) → `icon_ctx != 2`. Old code (skip disabled, field kept): **FAILED**
at the second assert — the icon kept arming. Icon-family regression tests re-run green:
fix_08_walls_live 3, cable_span_live 4, fix_context_hints_live 6.

## Test runs this session (each binary whole)

- `collide_column_census` (nf-view): 1 passed.
- `blast_cooks_off_grenades_live` (nf-game): 2 passed (red on the old code: 1 failed as above).
- `third_icon_kill_live` (nf-view): 1 passed (red on the old code: failed as above).
- `fix_08_walls_live` 3, `cable_span_live` 4, `fix_context_hints_live` 6 (nf-view) — icons
  unaffected.
- Full suite: `-p nf-game` ran to **completion, green** — every nf-game test binary passed (the
  filtered stream holds no FAILED/panicked line and the final doctest summary line printed; the
  changed code lives in nf-game: drone.rs, wire.rs, explode.rs, copter.rs). The `-p nf-view`
  half did not fit this environment: two 590 s windows went into compiling its ~150 test binaries
  and the linker hit disk-full bus errors at 87 % (cleared to 55 % by deleting the already-run
  nf-game test executables; a third window still ran out mid-compile). The nf-view binaries
  that exercise the changed code (scene.rs's icon loop, the census) ran green individually:
  `collide_column_census` 1, `third_icon_kill_live` 1 (+ its red proof), `fix_08_walls_live` 3,
  `cable_span_live` 4, `fix_context_hints_live` 6. The scene.rs skip only fires when a killed
  icon's kill channel is on — exactly one icon on the disc qualifies (07000041), which no other
  test's level carries.

## Left / hand-off

- **Car contact walk (0x131838)** — hand-off stub, MP-only content (see census). When MP car
  work resumes, port it together with the `Explode_Propagate` case-0x28 arm (both write
  CAR_INFO+0xf8).
- **Drone player-bump msg 0x22** [M `NDrone2_DealWithObjHit` 0x13fff0:108-126; the SM row
  "0x22 hit by thrown/picked object, 2 DSTATEs" docs/fields/drone-statemachine.md:167]: the port
  models the physical push (Jointy capsule) and the +0x26 latch's consumer is documented, but
  the message itself is never sent on contact. Not wired this session (SM-touching change, no
  safe red test inside the timebox); recorded as the deviation it is.
- **Player handler dead-state + ceiling tails** (0x1949a8:33-74, :168-175) — unmodelled, low
  impact.
- **Drone wall/ceiling anim flags** (anim+0x60 0x10/0x20/8) and the multi-hit last-hit pick —
  the port's single feet-ray stand-in is documented in drone.rs; the gameplay arms (bullets,
  hurt, smoke, bump) are all ported in their owning systems.
- **Draw work**: none owed by this audit — the ThirdIcon arm and every fix above is gameplay
  state; the killable icon's disappearance is not drawn in the port's renderer (icons have no
  mesh draw — the HUD context and the prompt are the visible surface, both handled).

## Live checks

1. **Cook-off in real play** (07000005): stand at the level start, drop/give a frag (weapon
   0x34), let it settle, then detonate anything within ~8 units of it — expect a second
   explosion at the frag one frame after the first. Address watch: `weapon_data` 0x2bf150 +
   0x34·0x10C +0x68 (read at `Bullet_handle_object_destruction` 0x12cd98:16).
2. **The 07000041 icon kill**: play 07000041 to the map-99 area, touch the TouchOnce trigger at
   (15.60, 0, −33.22) — expect channel 110 to fire (`watch 0x26fc90+0x6e 1`) and the ctx-2
   icon to stop arming BL+0x95F (read `M_CONTROL+0x95F` standing at (8.59, 0, −29.47): 0xFF
   after the trigger, 2 before).
3. **Drone bump msg 0x22**: on any level with a walking guard, walk into it — the game sends
   msg 0x22 (0x13fff0:126); confirm the 2 DSTATEs' reaction (docs/fields/drone-statemachine.md
   row 0x22) against the port's physical-only push, to decide if the arm is worth porting.

## Usage

not visible to me
