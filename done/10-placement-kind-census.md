# Placement kinds with no port arm

List every case of `parsemap_create_dynamic_objects` 0x1d04f0. For each kind, count placements per level and find the port's create arm. Report the table. Port any kind found on an on-foot level that has none, smallest first; queue-size ones go in the report's list with their counts.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/placement-kind-census`, report in docs/reports/placement-kind-census.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.

---
claimed-by: sandbox5 at 2026-11-07T20:28Z

---
## Report (done, 2026-11-07T21:54Z)

# placement-kind-census — report

Branch `s/placement-kind-census` from `origin/master` (10956badb). Task: every case of
`parsemap_create_dynamic_objects` 0x1d04f0, counted per level, each with the port's create
arm; port what an on-foot level needs and has none of.

## What was done

**The census** (crates/nf-view/tests/placement_kind_census.rs): `Level::parse` over every
`nf_view::dev::LEVELS` archive (34), counting the MAIN map's placements — the game's create
pass walks ONE map block per archive, the last block-0x1a parsed, which is the type-1 main
map's ([M] `parsemap_parsemap` 0x1d1cf0:65 defers to the state-2 epilogue over `DynamicPtr`;
`parsemap_block_map_data_dynamic` 0x1d1b68:88-96; the port's drone.rs attach comment says the
same). Per placement the gates mirrored from the caller: `(flags & 0xa000) == 0`
[0x1d1b68:88], the object resolved (by-hash through the archive hash table, by-index through
the main map's object list [0x1d1b68:76-86]), and the kind in the dispatch range — the jump
table bounds check is `(kind-0xf) < 0xf0` on the FULL flags word ([M disasm 0x1d04f4
`addiu v1,a2,-0xf` / 0x1d0518 `sltiu v0,v1,0xf0`], table at 0x304b20). No placement in any
archive has flags >= 0x100 while passing the spawn gate, so the port's `flags & 0xff`
convention misses nothing. Create-flag param 7 = 1 on the real load path
([M `ResetMap_Load` 0x1d6360:146]). Total counted: **6,050** placements, 0 unresolved.

**Every kind with occurrences has a port arm.** The table below is the test's own
classification (PORT_ARMS): a kind that occurs with a `nf-game none` row, or an unlisted
kind, fails the test — so the census stays enforced as the code evolves. The 0xF5/0xF8 pair
shares the AIVolume arm exactly as the switch does.

**Zero closures** (count 0 on every level, asserted in the test = the proof):
- 0x36 `Sub_Create` 0x1ad6c0 — 0/0 everywhere
- 0x37 `MiniSub_Create` 0x182130 — 0/0 everywhere
- 0xe0 `Creature_Create` 0x134ab8 — 0/0 everywhere
(matches the "Never-called proofs" comment in drone.rs and docs/systems/script-census.md;
now machine-checked, not just cited). 0xF8 also lands 0 (only the 0xF5 label occurs).

## The port: kind 0xDA (`Car_Create` 0x1305b0)

The census found exactly one kind whose placements occur and whose port arm was not wired:
**0xDA, 29 placements** — 4 each on the arenas 07000023/24/26/27/28/29 + 0700004b (Ravine's
four draw nothing: the level gate), **1 on 07000046** (an on-foot archive in LEVELS), 0 on
Sub Pen 07000025 [D, matches the mp_map_objects_live probe]. The port had the whole data
model (car.rs: gates, variant pick, activate/deactivate state machine, respawn) but NO
placement walk, and its create gate did not match the game.

Fixed/wired, both red-on-old-code verified on real archives:

1. **The gate** (`Car::create`): the game's off-gate is `MP-enabled && vehicles-Off` → no
   object ([M disasm 0x1305dc `lw v1, MPSettings+0x384`; `beqz v1` SKIPS the vehicles check
   when it is 0; 0x1305e8 `beql v0(=MPSettings+0x460), zero → return 0`]). The port had
   `!mp_enabled || vehicles_off` — an SP load could never create a car, while the game
   proceeds on a non-MP load with the option word standing (option 0 falls through the
   variant pick to the helicopter byte, the decomp's goto at 0x1305b0:34-40). Red proof:
   `car_placement_walk_live::sp_load_creates_the_07000046_vehicle` fails against the old
   gate ("the game creates the 07000046 vehicle on a non-MP load"), passes with the fix.
   The old pin in `car_state_machine::create_gates_match_car_create_0x1305b0` was updated
   to the decomp semantics (it had pinned the OR).
2. **The walk** (`CarSystem::create_walk`): the main map's 0xDA placements in placement
   order, one `Car_Create` each — the gates run per create (a gated-out placement draws
   NOTHING, not even the Random roll), the `7 < NumTanks` cap first, slot = created index,
   pose = `View_RotTransMatrix` of the placement, `spawn_matrix` = the same (the game backs
   the fresh pose up into `TankSpawns[slot]`: the Mat_Copy argument order is the
   const/non-const pattern shared with Car_Reset 0x132da8). The RNG draws only on option 3,
   only for a placement that passed the gates, in placement order (the shared stream).
   The MP host wiring calls it in `nf-view/src/lib.rs`'s match setup right after the
   0x25/0x26 registration walk and BEFORE `MP_Start`'s spawn picks (the game's parse pass
   runs first), storing the cars on `MpMatch.cars` (new field, struct + init literal).

### Remaining sim work on the cars (hand-off stub — NOT draw-verified)

- `Player_Activate`'s class-0x28 use-key arm (0x1a5170:142-145 — the sphere cast) →
  `Car::activate` (ported, unwired); the control walk's `Car::tick` (drive physics beyond
  `drive_step_heli` is car.rs's documented next-session work); `Car_Deactivate`'s caller
  glue. `MpMatch.cars` is storage only today.
- **Draw work (leave for the main agent's render):** the car object's renderer — the game
  draws `obj+0x90` plus the 3 turret sub-objects, all `View_SetDrawInAllViews`
  [M 0x1305b0:66-70]; the port's scene has no car instance. Render check spots:
  `--bond` at a Skyrail (07000024) tank spawn with NF_MP vehicles=Tanks, and the
  07000046 placement pose (SP path creates the heli variant there).

## The table

Per-kind totals (resolved/unresolved over all 34 LEVELS archives) with the port arm:

```
(kind, count, port arm) — 0x0f 555 drone.rs level_drones (Drone_Create 0x18a838); 0x20 443
breakable.rs (Break_Create 0x12c230); 0x21 33 world_fx.rs RipplesFx (0x1acc90); 0x22 20
ladder.rs (Ladder_Create 0x1ad3a0); 0x24 38 player.rs starts kind 1 (0x195c00); 0x25 252
mp/mod.rs spawn points (0x184f20); 0x26 198 mp/mod.rs MP objects (0x185990); 0x27 14
drone.rs GT walk + gt.rs (0x17ffa0); 0x28 29 sensor.rs (0x1af010); 0x29 33 useobj.rs
switches (0x1adcc0); 0x2a 74 sky.rs (0x1e79a0); 0x2b 3 world_fx.rs LeafGenFx (0x1acbd0);
0x2c 15 world_fx.rs LightningStrike (0x18cde8); 0x2d 18 player.rs starts kind 0 (0x195c00);
0x2e 24 lock.rs (0x1ae1f8); 0x2f 5 copter.rs (0x17d340); 0x30 4 monitor.rs (0x1b08b0);
0x31 12 fusebox.rs (0x1b0448); 0x32 129 drone.rs env walk rain boxes (0x17b5d8); 0x33 49
player.rs hints (0x1b3248); 0x34 15 gunimp.rs (0x131f20); 0x35 7 useobj.rs PC/Q-Worm
(0x1aeb58); 0x36 0 CLOSED; 0x37 0 CLOSED; 0x38 1 space_laser.rs (0x1aa8b8); 0x39 8
space_missile.rs (0x1a9ef0); 0x3a 57 gadget.rs GrapplePoint (0x198298); 0x3b 75 worldobj.rs
WorldObjects (0x18dbf8); 0x3e 15 wire.rs (0x1986c0); 0x3f 5 creepwall.rs (0x1b39c8); 0x41 6
worldobj.rs InverterTrigger (0x1b3040); 0x4a 26 world_fx.rs Apocalypse (0x1b1e98); 0x4e 4
light.rs level_lights 0x4e walk (0x1cc0e0); 0x71 15 worldobj.rs Rotor (0x18f4b8); 0xd2 5
worldobj.rs Shooter (0x17d030); 0xd4 2 world_fx.rs CloudFx (0x18d620); 0xd5 1 nf-view
scene.rs draw-side culling (0x1e7ce8; its Update IS backface culling); 0xd8 35 rigid_body.rs
(0x1d6bf0); 0xd9 172 nis.rs Scripts::new (0x1db9a8); 0xda 29 → PORTED (car.rs create_walk +
host wiring, this branch); 0xdb 443 door.rs (0x1baa28); 0xdc 3 triggers.rs (0x1b2c68); 0xde
54 searchlight.rs (0x1cb4f8); 0xdf 17 drone.rs env walk (0x17b968); 0xe0 0 CLOSED; 0xe1 189
props.rs (0x12c318); 0xe2 110 ss.rs (0x1a9c58); 0xe3 2 world_fx.rs FlickerFx (0x1acb00);
0xe4 64 hurt.rs (0x17e870); 0xe5 551 cover.rs (0x136ff0); 0xe6 351 cover.rs (0x137340);
0xe7 16 worldobj.rs ThirdCam (0x1bb160); 0xe8 16 triggers.rs LOAD_LEVEL (0x1b2950); 0xe9
129 aipoint.rs (0x137550); 0xea 346 triggers.rs TouchOnce (0x1b2e08); 0xeb 174 triggers.rs
Touch (0x1b2d38); 0xec 46 triggers.rs MultiplexIn (0x1b2f28); 0xed 21 triggers.rs
MultiplexSIn (0x1b2f88); 0xee 5 triggers.rs MultiplexOut (0x1b2fc8); 0xef 28 triggers.rs OR
(0x1b3020); 0xf0 378 nf-view scene.rs placed_pickup_from_placement + sim build (0x18dda0);
0xf1 79 spawners.rs walk in drone.rs attach (0x137b28); 0xf2 69 world_fx.rs EmitterFx
(0x17abc8); 0xf3 8 world_fx.rs TreeFx (0x17c200); 0xf4 1 triggers.rs MOVIE (0x1b2a08); 0xf5
223 aivolume.rs (0x149c60); 0xf8 0 (shares the 0xf5 arm); 0xf9 104 sound.rs SoundTriggers
(0x1b2628); 0xfa 44 wire.rs ThirdIcon (0x197070); 0xfb 15 worldobj.rs MusicTrigger
(0x1b30e0); 0xfc 113 corona.rs (0x18da00); 0xfd 5 props.rs CamSubject (0x1bc5d8); 0xfe 10
worldobj.rs Mine (0x17ce28).
```

Queue-size list (kinds with no arm left to port): **empty** — after the 0xDA walk every
occurring kind has a create arm. The driving/MP archives 07000023–07000029 were counted
(the MP matches run on-foot on them); they carry the arena subset {0x20 0x24 0x25 0x26 0x2a
0x34 0x3a 0xda 0xe1 0xe3 0xe4 0xeb 0xf0} — all armed.

## How it was verified

- `placement_kind_census` (nf-view): the table above, plus the zero-closure and arm-enforcement
  assertions. Ran green post-change: 1 passed.
- `car_placement_walk_live` (nf-game): the SP 07000046 vehicle (pose, slot, heli variant,
  no draw), the MP gates (Off → none and no draw; Tanks → 4, slots 0..4; Ravine → none, no
  draw). RED on the old gate, green after.
- `car_state_machine` 5/5 (its gate pin updated to the decomp), `car_registry_and_set_bits`
  7/7, `mp_never_mentioned` 6/6, `mp_map_objects_live` 4/4 (its 4-per-arena census holds).
- The nf-view test binaries and all 26 `walkreal_0700*_live` binaries compile clean against
  the changed lib. The full 4-package suite did not fit the sandbox window (the all-binaries
  build alone passed 10 min); every binary that links a changed code path ran green — the
  MP host path (the only live-only change) is compile-verified and its walk is covered by
  the headless tests above.

## What's left

- The cars' drive/activate/deactivate caller glue and `Car_Update`'s tank branch (car.rs
  header's next-session list), the class-0x28 use-key arm.
- The car renderer (draw work — the stub above).
- The census test is permanent: a new kind on the disc, or an arm vanishing, fails it.

## Usage

not visible to me
