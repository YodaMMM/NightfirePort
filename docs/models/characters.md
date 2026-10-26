# Models: every skinned character

Status 2026-10-12. Everything [D] (disc scans, `nf-extract` / the repo's own
`Level::parse`) or [M] (ACTION.ELF decompile) unless tagged [S]. The proof harness:
`crates/nf-view/tests/models_characters_live.rs` (6 tests, NF_ISO).

## 1. Completeness — the per-level inventory (the golden)

20 story levels, 515 drone placements, 88 distinct placed skins. Every placed skin resolves
post-load; the counts and hashes are pinned by `golden_every_story_level_s_placed_skins_resolve`
(numbers and hashes only). The level archives' own skin tables sum to 507 skins across the story
archives; 158 distinct skins exist on the whole disc [D scan of every type-3 entry].

The two exceptions the game itself creates, and their port handling:

| Level | Skin | Placements | The game's mechanism | Port |
|---|---|---|---|---|
| 0700000c | 0x05000090 | 7 | `AnimObjectNew` (0x1be910 disasm 0x1be958-0x1be9b0) substitutes **0x05000087** (0700000c's own skin) before the hashtable get | mirrored at load (`nf-view lib.rs`); the entry keeps the placement hash identity |
| 07000014 | 0x05000090 | 15 | the same, substitute **0x05000093** (07000014's own) | mirrored at load |
| 07000014 | 0x050000b1 | 11 | not requested raw: `NDrone2_DefaultInit` **0x14b300:82-95** rewrites the param-0 0x050000b1 to 0x0500008c in Drone+0xe0 on GameState+12 == 0x07000014, before the kind switch / captain block / `AnimObjectNew` request (:563-565 reads +0xe0) — 0x0500008c is a 07000014 own-archive skin [D]; captains then swap kinds 6/7 → 0x05000093/0x050000b2 [M :536-555]. The raw skin lives in 07000015/16's archives [D census: 07000016 entry 139 @0x8594a0 len 0x860, 07000015 entry 102 @0x609e60 len 0x860] and never registers on a fresh story run (the session skin table accumulates every archive loaded, `AnimLoadFile` **0x1bdf40** — first-wins, nothing frees between levels; the single-caller writer chain 0x1d4880 case 3 → 0x1bdf40 → 0x1bd500) — the rewrite is the game's only path for these placements. (Found by cite-14-skinless-drones; the earlier row here said "no substitution, renders skinless".) | the load merges the holding archive's maps, object hashcodes (first-wins), skins and skeletons for skins a placement DOES request; `CommonAnimFile` (0x30d860 [M 0x1d4a00 case 0 — the only reference, no writer; no type-4 call site exists, and FILES.BIN carries no common archive]) is 0 at every level-start snapshot [L ee_start_05/07000001/0700000a @0x30d860], so the common file is not the source — dead code in the retail build |

Dead references found (listed, not fixed — nothing to resolve against): 0700001b's cut-scene script
names skins **0x05050a17 / 0x05050ad2**, which exist on NO archive of the disc [D scan of every
type-3 entry, including the 0x0505 prefix]. The game's `AnimObjectNew` returns 0 for any hash
outside 0x05xxxxxx and for misses — those two actors cannot draw in the game either.

## 2. Exactness

- **Local poses are bit-exact against every EE snapshot**: `nf-extract pose-check` on
  `ee_start_05.bin` (07000005), `ee_start_07000001.bin`, `ee_start_0700000a.bin` — worst local
  0.00e0 on all three. The only nonzero world deltas sit on the one drone whose sAnimObject+0x58
  carries the live head-track/aim flags (0x40f) — the game's runtime look-at state, not a port
  error. G-024's world-matrix 4.6e-5 item is GAPS G-024's (closed).
- The two-bone blend and the palette are pinned GPU = CPU by `gpu_skin_live` (existing).

## 3. Tint / ambient and the texel check (G-042)

- The 0x800 TWEAK/ambient multiplier root cause is fixed (fix-render2, gs-leaf.md §1.1): 0x800
  objects multiply by tint × cel ambient >> 8. The source-side pin:
  `g042_the_bathhouse_civilian_skin_texels_are_not_white` — 07000002's civilian body skin
  0x05000034 ("polySurface2") decodes with non-white texels on every batch texture (≥ 8 checked);
  the old over-bright was the lighting path, not white source data.
- F3 (the dressing cut-scene, 07000006): re-verified on the current master — the suit actor
  (0x05000007) and the tuxedo actor (0x0500000d) both resolve and pose through the whole
  cut-scene; the mid-script swap mechanic (`Script_AnimStart` 0x1d8f40 kills the stream's old
  object) is in the port's `anim_start`. The reported vanish was the pre-fix-render2
  script-character resolution. Regression: `f3_both_dressing_actors_resolve_and_pose`.

## 4. Render spots (one per level, for visual checks)

Stand 3 units north of each level's first placed drone group, facing it:

| Level | Spot | What to look at |
|---|---|---|
| 07000001 | `--bond -4.9,7.5,-3.1,0.0` | guards 05000036/39/3a/4f |
| 07000002 | `--bond -18.1,6.6,35.7,0.0` | the bathhouse civilians (05000036/3a/34) — G-042 |
| 07000003 | `--bond 30.8,7.4,-8.7,0.0` | guards + 0500002b |
| 07000004 | `--bond -2.6,6.6,27.1,0.0` | the single 0500001e placement |
| 07000005 | `--bond 28.3,38.2,41.9,0.0` | the yard guards (0500002e/f/30) |
| 07000006 | `--bond 5.0,51.7,-65.9,0.0` | the castle interior guards |
| 07000007 | `--bond 44.3,45.2,-65.1,0.0` | the 11 party skins (58 drones) |
| 07000008 | `--bond 158.4,47.2,-92.4,0.0` | the estate guards |
| 07000009 | `--bond 1.6,-367.2,19.3,0.0` | the deep-level guards (05000027/28) |
| 0700000a | `--bond -34.2,2.2,18.7,0.0` | the restaurant guards |
| 0700000b | `--bond -6.4,49.0,12.4,0.0` | the train guards |
| 0700000c | `--bond -8.1,13.3,-201.4,0.0` | the power station — **0x05000090 → 0x05000087 substitute** |
| 0700000d | `--bond 159.1,6.2,-291.9,0.0` | the power station 2 |
| 07000011 | `--bond 16.3,49.0,8.8,0.0` | the Tower-2 guards |
| 07000012 | `--bond 10.1,2.4,17.7,0.0` | the Tower-2 B guards |
| 07000013 | `--bond 4.2,-367.2,47.0,0.0` | the Tower-2 C guards |
| 07000014 | `--bond 40.8,2.2,59.9,0.0` | the airbase — **0x05000090 → 0x05000093 substitute, 0x050000b1 → 0x0500008c rewrite [M 0x14b300:82-95]** |
| 07000015 | `--bond 50.1,42.6,-3.1,0.0` | EvilBase 2 (05000087/8c) |
| 07000016 | `--bond 2.1,7.9,-84.1,0.0` | EvilSilo — the 0x050000b1 source archive |
| 0700001b | `--bond 2.9,1.5,-3.0,0.0` | the space station (05000078/8f/ba) |

Cut-scene actors render via the ScriptPlayer paths already exercised by `fix_cutscenes_live`
(the truck, the dressing, the intros); the scene builds one character per script Anim stream and
poses it from the anim script's sequences.

## 5. Still open

- **The gondola riding inherit** is fix-objects2's leftover (script-ENTITY parts vs the truck's
  ANIM parts — the ground-motion path), not a model issue; listed here because it shows on a
  character (Bond on the carriage).
- The 0x05050a17/0x05050ad2 dead references (see §1) — nothing to do unless the driver wants them
  substituted by policy.
- The kind-4 pickup's label text (PICKUPINFO+0x34) — fix-objects2's leftover, render-side only.
