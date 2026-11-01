## Your task: `tripwires-shot-from-range`, tripwire mines can't be blown up from a distance

On the tripwire levels (0700000a, 0700000b, 07000014) shooting a tripwire mine from range does not set it off in the port. Find what the game does when a round hits a tripwire's mine/emitter [M the tripbomb class's hit/damage handler, Bullet_CollisionHandler 0x12ce80's class dispatch; D the tripwire placements' collision], port it with the address cited, and pin: a P2K round from 15+ units at a mine detonates it (blast + beam gone), a round that misses does not. Run the tripwire/trigger binaries you touch, every *0700000b* / *07000014* walk and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/tripwires-shot-from-range`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-01T04:29Z

---
## Report (blocked, 2026-11-01T05:41Z)


---
## Report from the worker (blocked, 2026-11-01T05:41Z)

# tripwires-shot-from-range — report

Branch `s/tripwires-shot-from-range` (from master 521d9bb2). Time window closed with one leg
landed and the second designed; the next message continues from this branch.

## What the game does (the evidence)

**The two "tripwire" object kinds.** A census of the named levels (the committed probe
`crates/nf-view/tests/tripwire_shot_live.rs::census_tripbomb_placements`, run with the disc)
shows **no placement kind 0xFE anywhere** on 0700000a (38 maps), 0700000b (51 maps) or
07000014 (55 maps). The 0xFE kind is `Mine_Create` 0x17ce28's only creator
([M] `parsemap_create_dynamic_objects` 0x1d04f0 case 0xfe, the only call site of Mine_Create in
ACTION.ELF per the decompile grep). So on these three levels the shootable "tripwire mine" is
**the placed laser tripbomb — the stuck class-5 grenade object** the player places with the
0x3a weapon (14's tripbombs are 0xF0 PICKUP placements, #1341/#1342 [D audit]). The 0xFE mine
class exists on 07000015/07000041 and that leg is landed here too (below).

**A placed 0xFE mine shot from range** ([M], all in ACTION.ELF):
- `Collide_PickObj` 0x1eae80 adds every object whose world sphere (obj+0x80 centre, obj+0x8C
  radius) the flight segment enters — `vecutil_calculate_closest_point_on_line` of the HITTEST
  segment; the mine's class 0x4B is not in the mask skips (0x400→classes 3/0x12, 0x100→2/0x11,
  0x200→5) and `Bullet_update` 0x12e7b0 arms HITTEST+0x9A = 0x20.
- `Collide_Intersect` 0x1ebaa8 tests the mine cel's own tris with the one-sided ray
  (`Intersect_RayGeom` 0x10a8c8:168 skips `n·dir >= 0`).
- `Collide_Update` 0x1ea398:160-234 mirrors each surviving node into the victim's `obj+0xD0`
  with `+0x58` = the bullet obj and `+0x51` |= 1; the walk stops at the first solid node
  (0x1ea398:216-229), so a nearer wall or guard takes the round instead.
- `Mine_Update` 0x17c930:31-45 walks `obj+0xD0` and explodes on **ANY** node with `+0x58 != 0`
  — no weapon filter, so a round from any range detonates the mine: `Camera_Shake` at radius
  ≥ 0.5, the Explode object from the mine's params (damage = p0, radius/shake/scale = p1,
  script = p5), `Control_ObjectReferenced(mine,1)`, `Script_Play`, and the mine deletes
  (`obj+0xFE` |= 1) [M 0x17cb7c-0x17cb88].

**A placed tripbomb (the stuck grenade) shot from range** ([M]):
- `Bullet_CollisionHandler` 0x12ce80's solid object-hit arm (disasm 0x12cff0-0x12d004) sets the
  round state 2 (dead) and, when the round's damage (`weapon_definition+0xC`) > 0, calls
  **`Bullet_handle_object_destruction` 0x12cd98** on the hit node's object.
- 0x12cd98: only a **class-5** victim (`obj+0xFF == 5`) with its record's `+0x68 & 0x4000` and
  `+0x70 & 0x80204` qualifies: the object deletes (`obj+0xFE` |= 1) and `Explode_Create` fires
  from the **victim's own record** (radius = record+0x08, damage = record+0x0C, script
  **0x0600_0052**), and if the record carries the laser bit **`+0x70 & 0x80000`** the beam
  object at `BU+0x38` deletes too — "blast + beam gone". Records [L ee_start_05.bin sweep, the
  fix_satchel_tripbomb_live header]: the tripbomb 0x3a carries +0x70 = 0x080100 (sticky 0x100 +
  laser 0x80000), the satchel 0x3b +0x70 = 0x300 (sticky + fuse 0x200) — both pass 0x80204.

## Landed (committed, 8f94a84c)

- `crates/nf-game/src/world.rs`: `ray_tri_t` is shared (pub(crate)) — the one-sided game ray
  [M 0x10a8c8:168].
- `crates/nf-game/src/worldobj.rs`: `Mine.bullet_hit` (own line) — the port stand-in for the
  mirrored per-tick HITDATA node; cleared at the collide-pass head each tick.
- `crates/nf-game/src/drone.rs`:
  - the mines' contact flags clear in `update_player_bullets`' head (the game's per-frame
    hit-list clear [M 0x1ea398's first loop]);
  - per flying player bullet, the **mine candidates**: AABB pre-reject then the one-sided ray
    vs the mine's cel tris, taken only in the nearest-solid contest against the world stop, the
    breakables and the guards ([M] the sort + first-solid rule above); the round dies on the
    take [M 0x12cff0-4] and the contact lands on the mine;
  - the `Mine_Update` explode arm reads the contact (`mine.bullet_hit || touching(...)`).
  - Phase note: the port's mines loop runs after the bullet pass in the same tick, so the
    explosion lands the same tick; the game's walk reads the list the frame after the collide
    pass ([M 0x1336f8:238]) — a documented one-tick stand-in, no test depends on it.
- `crates/nf-view/tests/tripwire_shot_live.rs`: the census probe (#[ignore], run and its
  output quoted above).

Ran (NF_ISO on the slim ISO, `--test-threads=1`): `fix_satchel_tripbomb_live` 6 passed /
0 failed; `interactables_07000015_live` 14/0; `interactables_07000041_live` 14/0 + 2 ignored;
nf-game `--lib` **593 passed / 0 failed** (the crate the change is in).

## Not landed — designed, with the cites (the next message's work)

1. **The stuck-gadget leg** (the actual 0a/0b/14 symptom): stuck grenades (the placed
   tripbombs/satchels) as bullet candidates in `update_player_bullets` — the sphere pick
   [M 0x1eae80:92-108 closest-point vs the world sphere], then on the take:
   `Bullet_handle_object_destruction` 0x12cd98's rule — record gate (+0x68 & 0x4000, +0x70 &
   0x80204), delete, the blast from the victim's record (radius +0x08, damage +0x0C, script
   0x0600_0052) via the existing `pending_blasts` tuple, and the 0x80000 records' beam dies
   with the object. The port's `Grenade` already models the placed tripbomb
   (`stuck` + `StuckGadget{normal, arm, armed}`; the beam is the `armed` state). Deliberately
   NOT landed half-done: the pick radius for the gadget's world sphere needs its [D] glist
   bounds (a wrong radius would detonate gadgets from stray rounds across the walk tests —
   a new-red risk I could not verify in the window).
2. **The pin test** (`tripwire_shot_live.rs`): place a tripbomb on 07000014 via the real stick
   path (the `fix_satchel_tripbomb_live` harness), then a P2K round from 15+ units (P2K = rec 2;
   its flight range [L record +0x1C] covers it) detonates it — blast script 0x0600_0052 plays
   [M 0x12cd98], the grenade leaves the list, the beam is gone; a round aimed ≥ 2 units off at
   that range does not detonate (sister). Plus the 0xFE leg's pin on 07000015/07000041's mines.
3. **The FULL suite** on the final tree with `tools/test/suite.py` and its total line. The
   runner could not stay alive in this sandbox (three attempts, killed between tool calls with
   zero output); the build + the focused binaries above are the verified surface. The next
   session should start `NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out 2>&1 &`
   in its first minutes (setsid did not survive either) and run it in foreground chunks if
   needed.
4. **Render check for the main agent** (visual): the exploded 0xFE mine's mesh is not hidden by
   the draw side yet (no consumer reads `Mine.exploded` in nf-view — the game deletes the
   object [M 0x17cb7c]); `--bond` spot on 07000015 near a `Tripbomb_3rd_*` placement before and
   after a detonation should show the mine gone. The stuck tripbomb's beam disappearing with
   the shot detonation is the same class of check.
5. Drone-fired rounds run the same `Bullet_CollisionHandler` in the game; the port's
   drone-bullet pass is the documented separate path (the established SP deviation), so this
   task ports the player-round side only.

## Live checks for the main agent

- `tools/trace/gdbwatch.py` on a placed tripbomb's BU+0x38 (the beam object pointer) while a
  P2K round hits the mine: expect the write `BU+0x38 → 0` after `Bullet_handle_object_destruction`
  0x12cd98 (the beam delete) and the object's obj+0xFE bit 0 set.
- `tools/trace/fieldprobe.py`: the stuck tripbomb's obj+0x80/+0x8C (the world sphere centre and
  radius) — the pick radius the port needs for the gadget candidate ([S] stand-in 0.5 today).

## Usage

not visible to me
