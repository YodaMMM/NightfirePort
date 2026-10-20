# Break kill pieces — glass shards and debris (playtest 1017-1139)

What a killed 0x20 breakable spawns. The user report: "it disappears, but shouldn't it like
break and shatter into like smaller pieces?" — origin/master removed the killed pane from the
draw (`obj+0xFE` bit 0) and spawned nothing. This is the sim-side piece list the draw layer
renders each tick: `crates/nf-game/src/shatter.rs` (`Shatter`), fed by `crate::breakable`'s
kill path. See also `docs/systems/breakables.md` (the damage/kill rules — unchanged here).

Two piece kinds, chosen by the break kind [M 0x12b970:78-141]:

- **kind 2 (glass)** — `ShatterPolyRecurse` 0x12b270 per source triangle of `psiGetTriList`
  0x11c9d8 (the shatter data `psiCreateBreakableData` 0x117418 built from the draw model's GIF
  stream at load [M 0x1d04f0:70-73]).
- **every other kind** — `Debris_Create_Loop` 0x1ed548: one debris object per third collision
  triangle record of every leaf box, dressed by `EffectInfo[hit material]`.

`Drone_BrokenObject` 0x138588 runs first on that path but spawns NO pieces — it is the guard
bark: on 07000001, when the killer was a player bullet (`hitdata+0x58` is a class-5 object whose
`+0xE0→+0x30` is `glb_players`) and drones exist, each health-positive class-0xC drone speaks
`DroneFunc_ChooseSFX(0x40, 0x43)` [M 0x138588:41-76]. Not ported (no speech side in the sim).

## The silent kill [M 0x12b970:59-63]

`Break_Kill` reads `cel+8` — the collision block (`parsemap_block_Coll_Data_New` 0x1d1810 puts
it there). **Zero → return after the dead bit**: no sound, no pieces. Only kills WITH a
collision block play the sound and spawn.

## Glass shards

`psiCreateBreakableData` 0x117418 walks the draw model's stream into:

- a face array (12-byte records: u32 draw word + 3×u16 vertex indices — strips emit
  `(i−2, i−1, i)` per kicking vertex ≥ 2 [M 0x117418:121-130]);
- a vertex array, 64 bytes per vertex: pos at +0x10, ST/UV at +0x20, RGBA as floats at
  +0x28..+0x37 [M 0x117418:90-156].

`psiGetTriList` 0x11c9d8 hands that to `Break_Kill` (count at `+0x30`, faces `+0x10`, verts
`+0x14` of the cel's gfx record). The port's source: the object's `GfxObject` batches —
`Batch::verts` positions/uvs decoded from the same stream, `Batch::texture` as the face's draw
word, faces `(i−2, i−1, i)` for kicking i ≥ 2 (the game's fixed strip order, not the draw's
parity alternation) [M 0x117418 + `crate::ps2gfx::Batch`].

`ShatterPolyRecurse(depth, verts, obj, hitdata, draw_word)` 0x12b270, per triangle:

- `centre = avg(verts) + obj+0x30` (the object's world position — the LOCAL mesh plus the
  placement translation, no rotation: the same rule the debris loop uses [M 0x12b270:146-151,
  0x1ed548:114-131]);
- `distSq = SqDist(hitdata+0x40, centre)` — the hit position (the collision contact, world
  [M `Collide_Intersect` 0x1ebaa8:334-338]);
- **leaf** when `longestEdge² ≤ clamp(half·distSq, 0.5, 100.0)` [M 0x12b270:156-162]
  (`half·distSq` floored at 0.5, the result capped at 100);
- **leaf spawn**: the three verts are re-centred on `avg` (local-relative); a 0xa0 shard record
  on `ShardList` (0x316dd8 [M 0x12bed0 `InitShards`]):
  - `+0x0c` = the face's draw word; `+0x10` lifetime = `REC_FRAME_RATE·25.0` (25 ticks)
    [M 0x12b270:205-209]; `+0x14` delay = `dist·REC_FRAME_RATE·15.0` (dist×15 ticks — shards
    peel outward from the bullet hole) [M 0x12b270:190-192];
  - `+0x20` pos = centre; `+0x30` rot = 0 [M 0x12b270:183-191];
  - `+0x40` vel = `hitdata+0x30 · min(0.1, 0.125/dist)` — the contact's normalised normal
    (Vec_Normalise into HITDATA+0x30 [M 0x1ebaa8:391]) scaled by 0.125/dist capped 0.1
    [M 0x12b270:199-211];
  - `+0x50` spin = `(Rand_FRand(1.0) − 0.5) / (distSq·10.0)` per axis — three draws
    [M 0x12b270:193-201];
- **split**: rotate so the LONGEST edge sits on v0–v1 (`max(e01,e20) ≤ e12` → `[v1,v2,v0]`;
  `max(e01,e12) ≤ e20` → `[v2,v1,v0]`; else keep) [M 0x12b270:214-347]; `M = lerp(v1→v0, t)`
  with `t = Rand_FRand(1.0)·0.5 + 0.25 ∈ [0.25, 0.75)` — one draw — lerping pos, UV and RGBA
  [M 0x12b270:348-372]; recurse on `[v0, M, v2]` and `[M, v1, v2]` (M's draw-word bytes come
  from v0 [M 0x12b270:373-473]). No depth cap in the binary (the entry never branches on
  `param_1` [M disasm 0x12b270-0x12b310]); splits shrink the longest edge, so the leaf rule
  terminates.

Per-tick `UpdateAllShards` 0x12b808 (skipped while paused — the port's Sim has no pause):
lifetime ≤ 0 → remove; delay > 0 → `delay −= REC_FRAME_RATE`; else
`pos += vel; rot += spin; vel += WldGravity · (REC_FRAME_RATE² · 4 · FRAME_RATE_MUL)` and
`lifetime −= REC_FRAME_RATE` [M 0x12b808:52-76]. WldGravity = (0, −9.8, 0) [L grenade.rs:145],
FRAME_RATE_MUL = 1 at the fixed 60 Hz.

Draw alpha (the draw layer's rule): `min(lifetime · FRAME_RATE · 10.0, 255.0)` per shard
[M `DrawAllShards` 0x12bef8:50-56] — 25 ticks of linear fade, 10 per tick.

## Debris

`Debris_Create_Loop(cel+8, hitdata, obj)` 0x1ed548 walks the collision block (`COLLDATA`:
`+0x00` box array, `+0x04` triangle records, `+0x0c` surface bytes, `+0x0c/3` vertex blocks,
`+0x14` box count [M 0x1d1810]). **Every leaf box** (the signed short at box+0xC negative =
the 0x8000 leaf flag) spawns records `i = tris.start, start+3, start+6, … < tris.end` — the
stride is 3 in the disassembly (`addiu $s3, $s3, 3` at 0x1ed638, back-edge recompute
`s0 = s3·8` [M disasm 0x1ed5f8-0x1ed838]) — one debris object per record
(`docs/systems/breakables.md`'s earlier "per collision triangle" was wrong; corrected).

Per record (8 bytes: 3 vertex halfwords + the plane halfword — `crate::collision::CollTri`):

- world verts = `leaf.origin + s16·leaf.scale + obj+0x30` [M 0x1ed548:114-125];
- centre = `avg(verts) + obj+0x30` → hitdata+0x40 [M 0x1ed548:126-131];
- hitdata+0x10 = the record's plane normal (`s16 · 2⁻¹⁴` — the same fixed-point normal
  `Intersect_RayGeom` 0x10a8c8:326 stores on a bullet hit) and hitdata+0x1c the next raw words
  [M 0x1ed548:132-139];
- `Debris_Create(hitdata, EffectInfo[(hitdata+0x50) & 0x3F], obj+0x20, 1)` [M 0x1ed548:140-143].

`Break_Kill` sets `hitdata+0x50 = 0x10` when its low 6 bits are 0; otherwise it is the killed
contact's surface byte (the hit collision triangle's material [M 0x10a8c8:326]).

`Debris_Create` 0x1aaa00, per object (the Loop passes count 1), gated on
`EffectInfo+0x74 != 0`:

- a class-0x10 object at the centre (`obj+0xFF = 0x10` [M 0x1aaa00:28]);
- `obj+0xE8 = Rand_FRand(0.1) + 0.4` [M 0x1aaa00:30-33] — one draw;
- `INFO+0x32 = 1`; `INFO+0x30 = (EffectInfo+0x81 >> 1) + Rand_Rand(EffectInfo+0x81 >> 1)` —
  the lifetime in ticks [M 0x1aaa00:34-37] — one draw;
- velocity `INFO+0x10..0x18 = (spread·0.5 · hitdata+0x10..0x18 + hitdata+0x20..0x28 ·
  Rand_FRandHalf(spread)) · REC_FRAME_RATE` — the record's plane normal times half the spread;
  the hit-velocity term is **zero** (`Break_Kill` runs `Vec_Zero(hitdata+0x20)` [M 0x12b970:140])
  but its `Rand_FRandHalf` draws are still consumed, three of them [M 0x1aaa00:38-49,
  `Rand_FRandHalf` 0x1e3810 = `max·rand01 − max/2` on the shared x/y streams];
- spin `INFO+0x20 = Rand_FRandHalf_Vec(EffectInfo+0x7c)` — three more draws, per axis
  `±spin/2` [M 0x1aaa00:50, `Rand_FRandHalf_Vec` 0x1dfef0];
- the gfx entity = `EffectInfo+0x74` (the debris model hash) [M 0x1aaa00:51].

Eight shared-RNG draws per debris object, in that order; the glass path draws only at splits
(one) and leaves (three). All through the shared generator (`crate::drone::Rng`, bit-exact).

The `EffectInfo` table [D ACTION.ELF 0x3037c0, 0x88 bytes per entry, 23 entries — ASCII
section headers follow entry 22]: `+0x74` u32 model hash, `+0x78` f32 spread, `+0x7c` f32
spin, `+0x81` u8 lifetime base. Entry 0x10 (the breakable default): model 0x0200012d, spread
6.2, spin π, base 0x37 → life = 27 + Rand_Rand(27) ticks. Entries 0,1,2,8,20,21,22 carry
model 0 → `Debris_Create` spawns nothing. Indexes ≥ 23 are past the table; the port treats
them as no spawn [*port choice* — the game reads garbage there].

Per-tick `Debris_Update` 0x1aac00: `INFO+0x30 < 1` → dead (`obj+0xFE |= 1`); else
`obj+0x106 −= INFO+0x32` (the fade byte, 0xff at create [M 0x1aaa00:31], exhausted at 255
ticks), `INFO+0x30 −= 1`, `pos += vel`, `vel += WldGravity · REC_FRAME_RATE²` (the `obj+0xF4`
gate is clear at create), `rot += spin` [M 0x1aac00:12-56]. The drawn rotation accumulates
`spin` per tick on `obj+0x50`; the fade byte is exposed as-is (no draw-side alpha read found
for class-0x10 objects — the main agent's render check owns the look).

## Port shape

- `shatter::Shatter { shards, debris }` on `Drones` (public), `update()` per tick next to the
  break passes; the draw layer reads the piece lists directly (per shard: local verts, world
  pos/rot/vel/spin, lifetime/delay, draw word; per debris: model hash, pos/rot, life, fade).
- `breakable::Source` built at `Breaks::create`: the collision leaves' step-3 records (local
  verts, plane normal, surface byte) for every breakable, plus the gfx triangles for kind 2.
  The kill ray-tests the records (Möller–Trumbore on the local segment) for the game's
  contact triple — hit position, normalised normal, surface byte [*port choice*: the game's
  contact comes from its collision pass; exact float parity is the live check's job]. No
  records → the silent kill.
- `Breaks::kill(idx, a, b, rng)` is the single kill path (bullet, guard step, and the net
  pass's direct edit all route through it); `bullet_hit`/`drone_step` grew the `rng` param.
- The 41% sound roll still consumes no draw (the documented breakables.md deviation).

## Live checks for the main agent

- A shot 07000001 vase: the debris count equals Σ over its collision leaves of
  `ceil((end−start)/3)` [M stride 3], each piece's model hash 0x0200012d, and the launch
  speed along the record normal ≈ 3.1 units/s (spread 6.2 · 0.5).
- A shot window: shard count = the draw model's triangle count when every shard spawns as a
  leaf (small panes; big ones split — count grows), peel delay ≈ dist×15 ticks from the hole.
