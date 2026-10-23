# Double-Cross objective items (0700 stages 01–04): the case keycard and the census

Objective items could not be taken at two spots. Build: ACTION.ELF
`e7ffdcb3…`. Tags: [M] decomp, [D] disc level-bin data, [S] suggested.

## The 07000003 keycard (the glass display case, --bond 5.90,5.97,-42.41,1.710,-0.458)

- **What it is [D]**: script **0x06000563** stream 1 carries an obj_type-0xF0 `Entity` command for the
  `Pheonix Keycard` object with params `[kind 2, id 0, rounds 0, switch 0x4E, sound 0xFFFF, respawn 0]`.
  `Script_CreateEntity` 0x1d8968's 0xF0 arm calls `Pickup_Create` 0x18dda0 at the stream's posed matrix
  with rest = 1 [M disasm 0x1d8bcc-0x1d8c20]. The spawned card's centre is (6.55, 5.92, −42.56) — the
  case interior. **kind 2 = key** (`Pickup_Handler` case 2 → `Player_EquipKey` 0x1a97a0, never refuses
  [M 0x18e650:217-219]).
- **The objective [D/M]**: switch 0x4E is the watch channel of obj 0x0400000a "Retrieve Dragon Safe
  contents" (the mission table 0x2a3f18 [M]). The take tail writes `switch_channels[+0x28] = 1`
  [M 0x18e650:330-333] — the keycard IS the writer of 0x4E. docs/levels/07000003.md's "the writers of
  0x4E are not on the disc" missed it because the spawn rides a script, not a placement.
- **How the game opens the path [M]**: the script player (0xD9 placement, **mode 0x40, mask 0x4**) sits
  at the case (6.70, 6.14, −42.55). `Player_Activate` 0x1a5170 sphere-casts head + look and dispatches
  `SP_Activate` 0x1dc338 on the case's class-0 object [M :121-122]; the mask bit 2 is the activation
  trigger [M `SP_Update` 0x1dbda8 trigger flags]. E on the case plays the script, the Entity fires, the
  pickup goes live, Bond walks it into the take test (`Pickup_Update` 0x18e148:203-214: distance
  radius+1.1, then `Collide_LineOfSight` mask 0x721).
- **Why the port refused**: three stacked causes.
  1. `script_pickup_slots` stored **reservation-relative** indices while `Drones.pickups` holds the
     placed pickups first — slot (0x06000563,1) pointed at pickups[0] (the first armour), so the drain
     (`Script_CreateEntity`'s 0xF0 arm, drone.rs) never found its reservation; the spawn fell through to
     the push path (a duplicate) and the reservation stayed a gone zombie. Fixed in scene.rs: the sim
     build offsets the slots by the placed count. [Port bug; the game has no slot table at all — every
     spawn creates a fresh object (M 0x18dda0).]
  2. The E press must be an **edge**: the port's SP activation consumes `ext+0x2C` per tick and a held
     use key restarts the script every tick, so the stream never advances to the Entity command. The
     real path (one press) works: the repro spawned the live card at (6.61, 5.92, −42.56).
  3. **The take is still refused by geometry — open.** The take-ray from Bond's sphere centre
     (6.15, 5.84, −42.31) to the card centre hits a **material 0x4** triangle of `aster_room` (m50[36])
     — the display case's front panel, x = 6.446, y 4.84–6.00, z −43.56..−41.64 — baked room geometry
     on the disc [D]. The take mask 0x721 skips ONLY surface classes 0xd/0xe
     [M `Collide_Intersect` 0x1ebaa8:311-313 — `(mat & 0x3f) − 0xd < 2`], and bond/card/panel all sit in
     room 4 → cell 29, so cel scoping does not exclude the panel either [M 0x1ec690:162-181's same-cel
     rule]. How the PS2 clears this ray (the case entity's animated pose replacing the baked panel in
     `Collide_Update`'s dynamic set [M 0x1336f8:238]?) is not derivable from the decompile alone — see
     the live checks below. The port pins everything up to the take; the take itself waits on that
     measurement. Not loosened: the port's `segment_take_scoped` is decompile-shaped and stays.

## Stage 01 ("a crossbow-like object standing on the floor")

Nightfire has **no crossbow weapon**: the weapon table's names ([D] `weapon_data` +0x38 labels via
USATxt.dat) list Phoenix **Samurai** 0x32/0x33 (a crossbow) — and **no Samurai object, placement or
script spawn exists in 07000001–04 at all** [D the full object/placement/script scan]. Stage 01's data
holds exactly 6 pickups (armour ×2 kind 3, ammo ×2 kind 1 id 6, Pickup 007 ×2 kind 6) [D], its scripts
spawn only fax/particle props (obj_types 0x31/0xd9) [D]. The object seen at (−0.34, 6.47, −6.22) is not
a pickup the game data can create; the nearest candidates are the meeting-room statics (the 0xDB shoji
door #14, G-050's known render query). Nothing to fix in the take path there; the render identification
is open.

## The sweep (01–04)

Every 0xF0 placement switch channel and every script 0xF0 spawn switch word, against each stage's
objective watch channels [D the census test]:

| stage | objective channels | script 0xF0 spawns | objective-item pickups |
|---|---|---|---|
| 07000001 | 0x47 | none | none |
| 07000002 | 0x48, 0x4b, 0x4d | none | none |
| 07000003 | 0x4e, 0x51, 0x53 | **0x4E (the keycard)** | the keycard |
| 07000004 | 0x54 | none | none |

## Test

`crates/nf-view/tests/doublecross_items_live.rs` (NF_ISO-gated):
- `the_case_use_spawns_the_keycard_through_the_real_path` — that spot, one E press, the live kind-2
  pickup with switch 0x4E at the case, exactly one (sister: no press → inert, ch 0x4E untouched).
- `taking_the_keycard_sets_the_objective_channel_and_the_monitor_completes_it` — kind 2 never refuses
  [M 0x18e650:217-219], `Player_EquipKey` sets the flag [M 0x1a97a0], ch 0x4E drives record 0x0400000a
  to state 4 through `Mission_MonitorObjectives` [M 0x1827d8] (sister: without ch 0x4E it stays pending).
- `the_objective_item_census_of_stages_01_04` — the table above [D].
