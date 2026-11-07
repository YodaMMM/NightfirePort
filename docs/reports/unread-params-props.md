# unread-params-props — placement-param sweep: pickups, breakables, gun mounts, hazards, effects, lights

2026-11-07, branch `s/unread-params-props` (from origin/master 10956badb). Sibling sweep to
unread-params-switches over the prop classes. Method: the dispatcher's class cases
(`parsemap_create_dynamic_objects` 0x1d04f0) → each class's `*_Create` read in the decomp with
the disassembly where the C drops arguments (the gcc2-EE ABI passes args 5-8 in t0-t3 and 9+ in
8-byte stack slots — that is where the decompiler loses them) → the port-side read set per
class → a census of every param index and raw extra word, per class per level, over all 34 dev
levels (`Level::parse` + `nf_view::dev::LEVELS`; the throwaway census/probe tests ran in
seconds and were not committed). Placement params are the record's (idx,val) pairs at +0x4C,
scattered by the block parser to `level_tag+0x2C+4k` [M parsemap_block_map_data_dynamic
0x1d1b68:70-75]; the raw words at +0x24..+0x44 are the quaternion/scale the dispatcher stages
for the cases that take them.

## One real gap fixed: the 0x4E placement lights read the wrong words

`light::placement_light` (the case-0x4E creation, called from the shared placement walk) read
the placement's RAW `extra[2..9]` words — the quaternion z and scale floats — instead of the
params block. The game marshals `Light_Create` from `level_tag+0x2C+4k` [M 0x1d0b8c-0x1d0c04:
`lw 0x2c(s1)` radius p0 with the uint→float `cvt.s.w` and the ≥2^31 expansion, `lbu
0x30/0x34/0x38(s1)` rgb p1/p2/p3, `lhu 0x3c(s1)` channel p4, life −1 / kind 0 / intensity 1.0
constants, `lhu 0x40(s1)` flicker period p5, `lhu 0x44(s1)` seed p6, `lhu 0x48(s1)` mask p7,
`lw 0x4c(s1)` sound p8 (0 → 0xffff); `Light_Create` 0x1cc3d8 stores them]. On disc the raw
words are (0,0,0,1)/(1,1,1) floats [D probe, 07000001's four lights], so every placement light
in the port was built with radius 0.0, black rgb and no flicker mask — inert. The only level
with 0x4E placements is 07000001 (four lights, params p0=6 p1=255 p2=121 p3=103 p7=1 [D]).

Fix: `placement_light` reads the params by index (the same uint→float radius conversion kept);
`LightList::create_full` gained the `sound` word (p8, 0 → 0xffff — silent on every level, the
update's play path stays unported). `crates/nf-view/tests/placement_lights_07000001_live.rs`
loads the real 07000001, runs the real creation path, and pins all four records (radius 6.0,
rgb [255,121,103], mask 1, permanent, sound −1) — RED on the old code (radius 0.0), GREEN now.

## Two real gaps found and left (traced, not ported in the window)

- **Mines (placement 0xfe) — the whole class is absent from the port.** `Mine_Create` 0x17ce28
  reads p0 → ext+0x1C (explosion size, zeroed when p1 < 0.5), p1 → ext+0x20 (blast radius:
  `Camera_Shake(p1, …)`, toucher ext+0xE8 = p1/15), p2 → ext+0x10 and p3 → ext+0x14 (bob rate
  and frequency, each ×0.0002), p4 → ext+0x18 (bob amplitude ×0.1), p5 → ext+0x24 (script hash:
  `Script_Load` + `Script_Play(…,0)` on detonation) [M 0x17ce7c-0x17cff0; the dispatcher passes
  only a0-a3 — the C's fifth "framelist" argument is a decompiler artifact, no t-register is
  read]. `Mine_Update` 0x17c930 bobs `obj+0x34` = base + sin(frame × ext+0x14) × ext+0x18 and
  detonates on a non-empty contact list: explosion (size p0, radius p1), camera shake, the
  script, retire. Census: 9 mines on 07000015 (p0=80, p1=10..15, p5=0x06000052) and 1 on
  07000041 (p0=50 p1=10 p2=50 p3=50 p4=10 p5=0x06000012) — the port creates, draws and
  detonates none of them. Follow-up: create from the placement walk, bob, the contact
  detonation chain (`Explode_Create` + `Camera_Shake` + the script hook), a real-level test
  red on the old code.
- **Hurt volumes: the count/rate/path triple (the state-1 spline flight).** `Hurt_Create`
  0x17e870 also consumes the placement's raw +0x40/+0x42 (u16 count, u16 rate) and +0x44 (path
  reference) via level_tag+0x02/04/08 → ext+0x0C (= count−1 or 0), ext+0x04 (= (count−1)/rate),
  ext+0x08 (the path; `Spline_Interp` in `Hurt_Update` 0x17e5d0's state 1 — the volume flies
  and hurts in flight). Params 0-5 themselves are ported completely (p5 one-shot included).
  Census: the triple is non-zero only on 07000046's seven hurt volumes (count 6, rate 0x5A=90,
  path words …000E/…000F/…0010) [D probe]. Not ported; needs the path-table plumbing first.

## Closed classes (census + decomp)

| class | game reads [M] | port | verdict |
|---|---|---|---|
| 0xf0 Pickup | p0..p6 (type, item, amount, out-channel, sound 0→0xffff, respawn, label) + dispatcher constants spin=1, despawn=0 [M 0x1d05a4-0x1d05ec, 0x18dda0] | p0..p6 (scene.rs) | complete; p7+ zero on every level |
| 0x20 Break | p0 kind, p1 health, p2 mask ~p2 [M 0x12c2a0-0x12c2f8] | p0..p2 (breakable.rs) | complete; p3+ zero everywhere |
| 0xe1 Destroy | p0..p6 [M 0x12c3b8-0x12c4c0] | p0..p6 (props.rs, countdown merged at master HEAD) | complete; p7+ zero everywhere |
| 0x34 GunImp | **no params** — the level_tag arg is dead [M 0x131f20-0x1320d4, no level_tag load; gun from MPSettings] | reads none | matches the game. Census p0 (0x2000025/0x200023b) p1 (103/96) non-zero on 0700000d + 5 arenas are DEAD DATA (shape of the static TurretGuns rows — model hash + weapon id — but no code consumes them) |
| 0xe4 Hurt | p0..p5 [M 0x17e8dc-0x17e964] | p0..p5 (hurt.rs) | params complete; the +0x40/42/44 triple is the gap above |
| 0xfe Mine | p0..p5 [M 0x17ce7c-0x17cff0] | nothing | the gap above |
| 0x31 FuseBox | p0 script, p1..p4 channels [M 0x1b0498-0x1b04e4] | p0..p4 (fusebox.rs) | complete; p5+ never read |
| 0x4e Light | p0..p8 [M 0x1d0b8c-0x1d0c04] | p0..p8 after this fix | fixed here |
| 0x21/0x2b/0x2c/0x32/0x4a/0xd4/0xe3/0xf2 effects | ripples p0/p1, leafgen none, lightning p0/p1/p2, cloud p0..p4, apocalypse p0..p4, flicker p0/p1, emitter p0..p5 | world_fx.rs reads exactly those | complete; census values within the read sets |
| 0xfc Corona | none [M 0x18da04-0x18da48] | n/a | census zero everywhere |
| 0x32 RainBox | none [M 0x18d844-0x18d87c] | precipitation volume | census zero everywhere |
| 0xd8 RB | p0 mass (0→3.0), p2 type, p3 carry-light → RB+0x154, p4 impact sound → RB+0x158 (0→0xffff) [M 0x1d6c7c-0x1d6e08] | init math reads p0/p2 (rigid_body.rs); p3/p4 are RB_Update/resting-contact reads | create-side complete; p4 = 0xffff (silent) on EVERY level [D census]; p3 = 1 on 15 RBs (01/02/03/04) rides the documented RB-evolution gap (docs/systems/levels.md §7.1) |
| 0x3b DynObject | p0 ch-A, p1 ch-B, p2 fade distance, p3 mode byte [M 0x18dc54-0x18dc80] | all four (worldobj.rs) | complete |
| 0x71 rotor | p0/p1 bytes [M 0x1d0b8c case 0x71] | n/a | zero on every level — closed by census |
| 0xd5 OneSided | none | n/a | closed |

## The census (per level, non-zero only)

```
07000001: pickup×6 [p0 p1 p2 p3 p4 p6] break×55 [p0 p1] destroy×11 [p2 p5] fusebox×12 lightning×11 [p1] light×4 [p0 p1 p2 p3 p7] ripples×7 [p0 p1] rainbox×7 rotor×2 rb×4 [p0 p2 p3 p4]
07000002: pickup×8 [p0 p1 p2 p4 p6] break×50 [p0] destroy×6 [p0 p2 p5] hurt×1 [p0..p4] fusebox×8 ripples×16 [p0 p1] rainbox×15 dynobj×1 [p1] emitter×1 [p0 p2 p4 p5] rb×21 [p0 p2 p3 p4]
07000003: pickup×5 [p0 p1 p2 p4 p6] break×97 [p0 p1] destroy×5 [p2 p5] fusebox×5 rainbox×9 dynobj×1 [p1] rb×3 [p0 p2 p4]
07000004: pickup×4 [p0 p1 p2 p4 p5] corona×2 ripples×4 [p0 p1] leafgen×2 rainbox×3 rb×2 [p0 p2 p3 p4]
07000005: pickup×6 [p0 p1 p2 p4 p6] hurt×13 [p0 p1 p3 p4] fusebox×4 corona×13 rainbox×18 dynobj×2 [p1 p3]
07000006: pickup×11 [p0 p1 p2 p4 p6] break×1 [p0 p2] destroy×6 [p0..p5] hurt×1 [p0 p1 p3 p4] fusebox×5 [p0..p4] rainbox×21 cloud×1 [p0..p4] rb×1 [p0 p2 p4]
07000007: pickup×3 [p0 p1 p2 p4 p6] break×8 [p0] rainbox×3 dynobj×9 [p0 p1]
07000008: pickup×8 [p0 p1 p2 p3 p4] break×3 [p0] destroy×4 [p1 p2 p3 p4 p5] hurt×3 [p0 p1 p3 p4] fusebox×15 [p0 p1] rainbox×14
07000009: pickup×3 [p0 p1 p2 p4 p6] break×17 [p0 p1 p2] fusebox×23 rainbox×12 rotor×1
0700000a: pickup×5 [p0 p1 p2 p4 p6] break×45 [p0 p1] hurt×5 [p0 p1 p3 p4] fusebox×13 rainbox×7 rotor×1
0700000b: pickup×5 [p0 p1 p2 p3 p4 p6] fusebox×25 ripples×5 [p0 p1] rainbox×17
0700000c: pickup×15 [p0 p1 p2 p4] destroy×12 [p2 p3 p4 p5] fusebox×12 dynobj×5 [p1 p2] rotor×1
0700000d: pickup×16 [p0 p1 p2 p4 p6] destroy×11 [p0..p5] gunimp×1 [p0 p1] fusebox×12 dynobj×5 [p0..p3] rotor×1 rb×3 [p0 p2 p4]
07000011: pickup×13 [p0 p1 p2 p4 p6] hurt×3 [p0 p1 p3] fusebox×1 rotor×3
07000012: pickup×18 [p0 p1 p2 p3 p4 p6] break×97 [p0 p1 p2] destroy×1 fusebox×15 rotor×1
07000013: pickup×7 [p0 p1 p2 p4 p6] break×33 [p0 p1 p2] fusebox×2 rotor×1
07000014: pickup×32 [p0 p1 p2 p4] break×3 [p0 p1 p2] destroy×48 [p0..p6] fusebox×11 [p0..p4] rainbox×1 apoc×11 [p0..p3] dynobj×45 [p0 p1] emitter×35 [p0 p2 p5]
07000015: pickup×12 [p0 p1 p2 p4 p6] destroy×11 [p0..p5] hurt×9 [p0 p1 p3 p4 p5] mine×9 [p0 p1 p5] fusebox×1 corona×76 apoc×4 [p0..p3] emitter×16 [p0 p5]
07000016: pickup×17 [p0 p1 p2 p4 p5 p6] destroy×7 [p2 p5] hurt×2 [p0..p4] fusebox×31 corona×21 apoc×2 [p0..p4] dynobj×5 [p0 p1]
0700001b: pickup×7 [p0 p1 p4] destroy×6 [p1 p3 p4 p5] hurt×2 [p0..p4] fusebox×129 apoc×8 [p0 p1 p2] dynobj×1 [p1]
07000023: pickup×20 [p0 p1 p2 p4 p5] destroy×10 [p2 p5] gunimp×2 [p0 p1] fusebox×10
07000024: pickup×28 [p0 p1 p2 p4 p5] destroy×8 [p2 p3 p5] gunimp×2 [p0 p1] fusebox×9
07000025: pickup×24 [p0 p1 p2 p4 p5] destroy×11 [p2 p5] gunimp×2 [p0 p1] hurt×6 [p0 p1 p3 p4 p5] fusebox×18
07000026: pickup×18 [p0 p1 p2 p4 p5] break×9 [p0] destroy×6 [p2 p5] gunimp×2 [p0 p1] hurt×4 [p0 p1 p3] fusebox×6
07000027: pickup×19 [p0 p1 p2 p4 p5] break×8 [p0] destroy×12 [p2 p5] gunimp×2 [p0 p1] fusebox×12
07000028: pickup×17 [p0 p1 p2 p4 p5]
07000029: pickup×20 [p0 p1 p2 p4 p5] break×1 [p0] destroy×8 [p2 p5] gunimp×2 [p0 p1] fusebox×4 flicker×1 [p0 p1]
07000041: pickup×1 [p0 p1 p2 p4] break×16 [p0 p1 p2] mine×1 [p0..p5] fusebox×4 lightning×4 [p1] flicker×1 corona×1 ripples×1 [p0 p1] leafgen×1 rainbox×2 cloud×1 [p0..p4] apoc×1 [p0..p4] dynobj×1 [p0 p1] rotor×4 oneside×1 rb×1 [p4]
07000046: pickup×2 [p0 p1 p2 p4] hurt×7 [p0 p1 p3 p4 p5] fusebox×6 [p0 p1 p2] emitter×17 [p0]
07000048: fusebox×1
07000049: (no prop placements)
0700004a: fusebox×1
0700004b: pickup×28 [p0 p1 p2 p4 p5] destroy×6 [p2 p5] gunimp×2 [p0 p1] hurt×8 [p0 p1 p3 p4] fusebox×6
```
Raw extra words: e3..e6 are the quaternion w / scale triple (1.0 floats) on nearly every
record; the only class reading raw words beyond those is 0xe4 Hurt (+0x40/42/44, the gap
above) — plus the +0x24 quaternion the 0x27/0x34/0xf2 creators take (already ported).

## Runs

- `placement_lights_07000001_live` 1/0 — RED on the old code (radius 0.0 vs 6.0), green after the fix.
- nf-game lib 688/0/5ign; lighting_live 10/0; interact_07000001_live 5/0; walkreal_07000001_live 8/0;
  g049_blast_light_live 1/0; npc_doors_live 2/0/1ign; walkreal_07000002 6/0/1ign; walkreal_0700000d 3/0/1ign;
  walkreal_07000003 6/0/1ign; walkreal_07000004 10/0; walkreal_07000005 14/0; walkreal_07000006 6/0/1ign;
  walkreal_07000007 6/0 (five of the 03..09 batch finished before the window cut; all pass).
- Not run: the full serial suite and the remaining walkreals (08, 09, 0a-0c, 11-16). Justification:
  the only changed function is `placement_light`'s 0x4E branch, unreachable on every level but
  07000001 [D the census — no other level carries a 0x4E placement], and the RNG stream is
  unchanged (the seed source stays 0 → the same single `Rand_Rand(0)` draw).

## Live checks for the main agent

- 07000001: the four 0x4E lights should now render as warm (255,121,103) point lights of radius
  6 at their placements; before this fix they drew nothing. Any bond spot near one of the four
  placements shows it.
- 07000015: the nine 0xfe mines bob in the game and detonate on contact (blast + 0x06000052
  script); the port has none — a playtest walk over a mine placement shows the gap.

## What's left

- The mine class (0xfe) port — full trace above; biggest remaining prop gap.
- The hurt state-1 flight (count/rate/path triple) — needs the path-table plumbing.
- The RB evolution port (p3 carry-light, p4 impact sound ride it) — pre-existing tracked gap.
- The full serial suite on the final tree (window too short here).

## Usage

Environment figures for this session: the three decomp-research subagents each reported their
own counters, read from their return messages: "Tokens -> prompt:153957 completion:6682
total:160639", "prompt:127391 completion:18161 total:145552", "prompt:148391 completion:4712
total:153103". My own session's input/output tokens, context and cost: not visible to me.
