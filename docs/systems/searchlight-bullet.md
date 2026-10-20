# Searchlight bullets: the head's contact list (U-041 ported)

What happens when a player bullet reaches a searchlight. Everything here is [M] unless tagged:
ACTION.ELF addresses from the decompile and the disassembly (`work/decomp/ACTION.ELF`, capstone
dumps in this task's session). Port side: `crates/nf-game/src/searchlight.rs` (`bullet_check`,
`Shot`), `crates/nf-game/src/drone.rs` (`sl_shots`), `crates/nf-formats/src/weapon.rs`
(`bullet_contact_damage`), `crates/nf-view/src/scene.rs` (`frame_lights`' node-death gate).

## The game's path, end to end

1. **The bullet is a class-5 object with a HITTEST** (`obj+0xD4` = instance+0x50, set at
   `Bullet_init` 0x12c5a0:102). Every flying tick its HITTEST is refreshed by the
   `Bullet_update` 0x12e7b0 tail (disasm 0x12efd8-0x12f188):
   - `+0x90` = `weapon_data[w]+0x0C` — the contact damage. When `weapon_data[w]+0x08` as f32
     > 0.0, the id switch overrides: ids 0x2A/0x2C-0x2F (the impact-explode rockets) → 1.0,
     ids 0x2B/0x34-0x37/0x3A-0x40 (the throwables) → 0.0. (Disasm 0x12f05c-0x12f0b8; the jump
     table at 0x2f4320 covers ids 0x2A-0x40, outside ids take the record value.)
   - `+0x94` = 0x201 (the pick mode; 0x101 when the raw tick step < 0.125 — the bullet obj's own
     0.1 world radius floored at 0.25, halved — and the weapon id is not one of the
     always-segment ids 0x2C-0x31/0x62/0x6D; disasm 0x12f0c8-0x12f134, the mode store 0x12f134/
     0x12f154). Dead bullets (state ≥ 2) store 0 — the pick is off (0x12f03c).
   - `+0x98` = 0 (the mask bits: no class skipped), `+0x9A` = 0x20 when `+0x6C & 0x800 == 0`.
   - The segment: `+0x20` = obj+0x30 (the pre-move position), `+0x30` = obj+0x30 + dir·step
     (the post-move position, step floored at 1.0 and clamped to the range), `+0x40` = dir,
     `+0x8C` = max(step, 1.0).
2. **`Collide_Update` 0x1ea398** runs per frame: pass 1 (:25-52) frees every object's `obj+0xD0`
   list to the node pool 0x34b7b0 and stamps `hittest+0x70` = owner; pass 2 (:72-240) runs
   `Collide_Pick` 0x1eb320 per enabled HITTEST (`+0x94 != 0`) — for the bullet's mode 0x201,
   `Collide_PickObj` 0x1eae80 (:92-141) tests every candidate object of the straddled cels:
   **the closest point on the flight segment to the object's world sphere centre (`obj+0x80`),
   hit iff its squared distance < (`obj+0x8C`)²** (the closest-point helper
   `vecutil_calculate_closest_point_on_line` 0x1df7e0 clamps t to [0,1]). A hit pushes a node
   {+0xC param, +0x58 = the hit object} onto the PICKER's list; the redistribution (:160-215)
   then copies each picked node with a hit object onto **the hit object's own `obj+0xD0` list**
   as {damage f32 +0x8 = picker hittest+0x90, param +0xC, flags +0x51 |= 1, hit data, attacker
   obj +0x58 = the bullet}. Pass 3 (:54-70) calls each class's collide handler.
3. **The world sphere** is `Control_BuildWorldSph` 0x133070's product: centre = the object's
   matrix · `glist+0x10` (the local bounds centre), radius = `glist+0x1C` · |obj+0xE8| — with
   obj+0xE8 = 1.0 from `control_init_object` 0x134630:17. The head is pickable:
   `Searchlight_Create` 0x1cb4f8:73-76 sets obj+0xD8 = the part's glist (the 0x1eae80:88-90 gate).
4. **`Searchlight_Update` 0x1cb900 is the class-0x22 UPDATE** (`control_funcs` 0x26f890, entry
   0x22: update = 0x1cb900, collide = 0) — it runs after `Collide_Update` in the frame and reads
   the fresh contacts. The tail walk (:406-462) runs in states 0/1/3 (state 2 returns on every
   path before it, :373-393; the state-3 relight jumps straight to the walk, :399-404): every
   entry whose attacker's class byte (`+0x58 → obj+0xFF`) is 5 **and whose damage float (+0x8)
   is > 0.0** breaks the light [M :411-417]:
   - state := 2 [M :418];
   - the spot's psiLight node freed (`LList_Remove` 0x2c74cc / `psiLight_Delete` / back to the
     0x2c74c0 pool) and the SPOT object deleted (`obj+0xFE |= 1`), links[2] := 0 [M :419-439];
   - the beam and corona objects deleted, links[3]/links[4] := 0 [M :440-453]. **The mount
     (`links[0]`) is untouched** — it stays drawn and linked;
   - `Control_NextLOD` — the damaged-model swap (obj+0xD8 ← glist+0x40 [M :454]; visual, the
     port keeps the intact model — a renderer note);
   - `Sound_Play3D(0xE6)` (the glass break) at the head [M :455-456];
   - the walk continues: a second qualifying contact re-runs the block (state 2 again, another
     0xE6) [M :457 → the loop head].
   Non-bullet attackers (class ≠ 5) and zero damage never break: the entry is skipped [M :411].

Who reads the contacts: only the HEAD (kind 1). A head-less update returns at
`if (*psVar2 != 1) return` [M 0x1cb900:89], so a contact landing on a MOUNT object (its own
class-0x22 sphere is picked the same way) is never read — shooting the mount does nothing.

## The port

- `Searchlights::update(.., shots: &[Shot], ..)`: `Shot` = one flying player bullet's per-tick
  pick {from, to, damage, segment}. `Drones::update_player_bullets` builds them from the same
  segments it pushes into `bullet_paths` (A = pos, B = pos + dir·max(step,1) range-clamped —
  the game's own floor/clamp), with `WeaponData::bullet_contact_damage(w)` and the mode
  (segment, or the 0x101 point arm when step < 0.125 and the id not in 0x2C-0x31/0x62/0x6D).
- `Searchlights::bullet_check` = the walk: the head's world sphere posed from the part's
  `sph_centre`/`sph_radius` (the map object's glist bounds — the same values the doors' live
  sphere uses) through the part's matrix; the closest-point segment test (or the point test
  with the 0.25 + radius margin); the break block per qualifying contact (state 2, spot/beam/
  corona hidden + links[2..4] cleared + `part.light = None`, mount untouched, the 0xE6 sound).
  The port's `drawn = false` stands in for the game's `obj+0xFE |= 1` deletion (the objects
  have no separate lifetime in the port); `Control_NextLOD` is not ported (visual).
- The psiLight death: the port's tracked light record (kind 2, `track` = the spot part index)
  resolves per frame; `frame_lights` marks a record whose tracked part lost its node
  (`part.light == None`) inactive, so the renderer stops picking it — the game's
  `psiLight_Delete` equivalent. This also covers the channel-kill path (which already set
  `part.light = None`).
- The contact list itself is per-frame in the game (freed each `Collide_Update` head) — the
  port's `sl_shots` is cleared and rebuilt every tick the same way.

## What is NOT ported (documented gaps)

- Drone/guard bullets: the game's contacts come from EVERY class-5 bullet (guards shooting the
  lights break them too); the port's feed is the player's bullets (`player_bullets` →
  `sl_shots`). The guards' own flight path (`Drones::bullets`) is not wired in.
- Grenades/throwables as class-5 flyers: the port models them in `crate::grenade`, outside
  `bullet_paths`; their forced-0 damage would not break anything anyway.
- `Control_NextLOD`'s damaged model (visual, needs the glist+0x40 model swap in the renderer).
- The kill-path tail (the game still runs the walk after the channel kill — searchlight.md port
  mismatch 7) and the relight-tick part updates (mismatch 4) are pre-existing, unchanged here.

## Tests

- `crates/nf-game/src/searchlight.rs` (unit, no ISO): the walk breaks on a damaging contact with
  the mount kept; a mount-only contact does not break (the U-041 symptom — the old stand-in
  tested the mount's box); the gate is the damage float (0.0 never breaks); a broken head
  ignores further contacts; the slow point arm reaches 1.2 with a 1.0 sphere (the segment arm
  misses — the sister).
- `crates/nf-game/tests/searchlight_live.rs` (ISO-gated, 07000005): the bullet leg now aims
  through the head's posed world sphere (the old aim "through the head's box" was the stand-in's
  target — fixed with the address, see the report).
- `crates/nf-view/tests/searchlight_bullet_07000006_live.rs` (ISO-gated, the real path): Bond
  fires at a real 07000006 courtyard head through `Sim::step_bond` — the break effects match the
  game's list; the mount-only shot does not break (fails on the stand-in).
