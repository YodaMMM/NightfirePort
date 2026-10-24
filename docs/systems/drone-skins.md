# Drone skins — the G-021 lookup, end to end

Every fact below carries its evidence tag: [M] decompile (function + address), [D] disc bytes,
[L] EE snapshot / live PCSX2, [S] suggested. Addresses are ACTION.ELF unless noted.

## The loader chain (how a skin registers)

1. **Containers.** `LoaderLoad` 0x1d4a00 builds the name `"%s%8.8x.bin"` (prefix 0x30d018) from
   the hashcode with its 0x00F00000 nibble rewritten (loader type 0/3 → `|0xF00000`, type 4 →
   `CommonAnimFile`, type 5 → `|0xD00000`), opens it with `psiFileOpen`, and walks its directory:
   state 1 reads the header (dir size + entry count), state 2/4 walk `{u32 length, u8 type, char
   name[]}` entries, parse `DirFileHash` from up to 8 hex digits of the name, and call
   `LoaderProcess` 0x1d4880 per entry [M 0x1d4a00 states 0-5; docs/formats/level-bin.md].
2. **Skins.** `LoaderProcess` case 3 hands a type-3 sub-file to `AnimLoadFile(DirFileHash, 0)`
   0x1bdf40 [M 0x1d4880 case 3]. For a 0x05 hash it gates on `MP_NeedSkin` 0x120c48 (1 in SP —
   `GameState+0x84` == 0 [M :10-12]), serves the preloaded sub-file buffer through the
   single-file-mode `psiFileLoad`, registers it **first-wins** (`hashtable_additem` only when
   `hashtable_getitem` misses) and parses it with `AnimProcessSkinData` 0x1bd500, which
   `hashtable_modify`s the raw data into the 0x40-byte skin record (main list at rec+4 with the
   celglists, extras at rec+8, datums at rec+0x20, skeleton byte at rec+0x30) [M 0x1bd500:84-205].
3. **The sleeve rule.** `AnimProcessSkinData` :159-166: a MAIN entry whose byte 2 carries bit
   0x10 becomes `AnimSleeveGetEntity(0)` 0x1c6590 — the first present entity of the fixed
   `SleeveEnts` list @0x2c6ed8 — before the celglist lookup [M]. No *placed drone* skin carries a
   0x10-flagged main entry [D census, fix-missing-drone-skins]; the rule fires on view-model fp
   skins only.
4. **Nothing frees skins.** The only hashtable deletions are `LoadableLoad`/`LoadableReload`
   0x1d69e8/0x1d6ab0's `hashtable_delete_hashtype(0x4000000)` — 0x1ca520 compares each entry's
   hash **top byte** against the argument's, so only 0x04 sequences die — and
   `Script_FreeFromHashTable` 0x1daec0's individual scripts [M]. Skins (0x05) accumulate over the
   whole session: front-end container (boot) + every container the flow loaded since.
5. **`CommonAnimFile` (LoaderLoad type 4) is dead code** — variable 0x30d860 has no writer, no
   type-4 call site exists, the value is 0 in every snapshot, and FILES.BIN carries no common
   archive [M store scan + D/L in fix-missing-drone-skins].

## The per-placement request

`NDrone2_DefaultInit` 0x14b300 puts the skin hash in `Drone+0xe0` (the placement param 0) and:

- **The 07000014 rewrite** :82-95 — with `GameState+12` == 0x07000014, a param-0 skin
  0x050000b1 is rewritten to 0x0500008c **before the +0xe0 store**: the kind switch, the
  captain block and the `AnimObjectNew` handoff all read the rewritten value, so the
  placement never asks the skin table for 0x050000b1 (found by cite-14-skinless-drones; the
  earlier version of this doc missed it and concluded the b1 placements render skinless).
  0x0500008c is a 07000014 own-archive skin [D] — kind 7, the same row b1 maps to in the
  kind table [M the 0x500008c/0x50000b2 cases].
- **Captain swap** :536-555 — when the difficulty-gated behaviour property marks the drone a
  captain (`+0x19`: property 9 at GameState+0x28 == 1, property 5 at == 2, property 8 at == 3;
  the decompiler's `_40_4_` names are DECIMAL offsets — hex +0x28, 0x2a3790, the difficulty
  [M docs/systems/progress.md; boot default 2, 0x1c7e78:25]), the drone's **kind** (from
  `drone_setup`'s skin→kind table, nf-formats/drone.rs:129) picks a fixed skin: kind 1 →
  0x05000038, 2 → 0x0500002f, 3 → 0x05000006, 6 → 0x05000093, 7 → 0x050000b2; other kinds keep
  the placement's [M the switch's full range]. On 07000014 the swap runs on the REWRITTEN +0xe0
  (a b1 placement's kind-7 captain ends at 0x050000b2, a non-captain at 0x0500008c).
- **Substitution** — `AnimObjectNew` 0x1be910:33-43 (disasm 0x1be958-0x1be9b0): with
  `GameState+0xC` (0x2a3774) == 0x0700000c, a request for 0x05000090 becomes 0x05000087; on
  0x07000014 it becomes 0x05000093. No other pair substitutes (full-range constant scan) [M].
- **The get** — :44-46 `hashtable_getitem` 0x1ca5b0 (a binary search over `t_hashtable` keyed by
  the hash top byte, `HashTypeRanges`). **Miss → return 0**, and `NDrone2_DefaultInit`
  :563-565 discards the return: the drone keeps no skin record and `AnimObjectNew`'s
  `Control_CreateObjEx` loop (:154-172) — the only creator of the character meshes — never runs.
  The placement renders **skinless** (the AI, weapon and sounds are unaffected) [M].

## The disc carriers [D]

Scan of all 99 FILES.BIN `.bin` containers' type-3 sub-files (2026-10-17, scripts/
probe_skin_carriers.py against the slim ISO):

| hash | carriers |
|---|---|
| 0x05000090 | 07000049, 0700000d, 07000016, 07000015 |
| 0x05000087 | 07000049, 0700000d, 0700000c, 07000014, 07000016, 07000015 |
| 0x05000093 | 07000049, 0700000d, 0700000c, 07000014, 07000016, 07000015 |
| 0x050000b1 | **07000016, 07000015 — only these two** |

Story order (nf_game::mission::LEVELS): …, 11, 12, 4a, 13, **14**, 15, 16, 1b. So on a fresh
story run the session table at 07000014's load holds the front-end (0x07000048, boot) plus
05…13/4a — **no carrier of 0x050000b1**. But the placements still render: the render request is
not the raw param — the :82-95 rewrite turns every 07000014 b1 request into the own-archive
0x0500008c before the table lookup, and a kind-7 captain's swap lands on 0x050000b2 (also
own-archive). The 22 0x05000090 placements (7 on 0700000c, 15 on 07000014) resolve
through the substitution into each level's own archive. [D carrier census with entry offsets:
`drone_skins_all_levels_live::b1_carrier_census_and_the_skinless_placements`, cite-14-skinless-drones.]

## The port (drone-skins-g021 + cite-14-skinless-drones)

- `nf_view::load`'s G-021 merge is **session-gated**: a foreign archive is a source only if the
  session could have loaded it — the front-end 07000048 or a story level BEFORE the current one
  (mission::LEVELS order). 07000014's raw 0x050000b1 therefore merges nothing (crates/nf-view/src/
  lib.rs, the G-021 block) — and does not need to: the placements request the rewritten
  0x0500008c, which the own archive carries.
- The sim's `Drone::new` applies the :82-95 rewrite to the placement param before every use
  (kind, weapon, `dr.skin`), and `captain_skin` (nf-game/src/drone.rs — the pure form of
  `Drone::captain_setup`'s 0x14b300:536-565 rule, rewrite included) feeds `Scene::build`'s
  character pass, which rebuilds a placement's instances whenever the final +0xe0 hash differs
  from the raw param (the rewritten b1 placements and the swapped captains) from the final
  skin's mains. A final skin the table lacks draws nothing — the game's miss (crates/nf-view/
  src/scene.rs, the drone_skins block).
- `captain_skin_of_kind` is the one source for the kind→skin table, shared by the sim's
  `captain_setup` and the draw-side probe.
- The census test `drone_skins_all_levels_live.rs` pins, per story level: the raw-missing param
  set, the per-drone resolved hashes, the 07000014 b1 drones' effective skins (the rewrite
  0x0500008c or a kind-7 captain's 0x050000b2, skinless empty — cite-14-skinless-drones), the
  substitution's content equality, and the sleeve rule.

## What is not traced here

- The captain's held weapon follows the RAW skin in the sim (`weapon_id` runs in `Drone::new`,
  before `captain_setup`) and the swapped skin in the draw — whether the game re-derives the
  weapon after the swap is untraced (see the report's live checks).
