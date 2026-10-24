# cite-14-skinless-drones — the 07000014 skinless drones, settled from the disc and the decompile

Task: a [D] census behind `missing_models_census_live.rs`'s `skinless_drones == 5` pin on
07000014 ("the 0x050000b1 non-captain drones are the shipped game's own skinless renders
[D drone-skins-g021]"), whose only evidence was a doc name — then either change the test to the
game's rule, or mark it `// [S] unverified`.

**The claim flipped.** The census settled the [D] facts exactly as the old doc said, but the
pin's conclusion contradicted the decompile: the game REWRITES the request before it ever
reaches the skin table, so 07000014's b1 drones are **not** skinless. The pin is now
`skinless_drones == 0`, and the port was missing the rewrite — that gap is fixed.

## 1. The [D] census

Assertion-free diagnostic committed on the branch:
`crates/nf-view/tests/drone_skins_all_levels_live.rs::b1_carrier_census_and_the_skinless_placements`
(`--ignored --nocapture`; output in the WORKLOG and below).

- **Carriers of a 0x050000b1 type-3 sub-file across all FILES.BIN containers** (a type-3
  sub-file starts with its u32 LE skin hashcode — the same read the loader's `DirFileHash`
  walk serves): exactly two —
  - `07000016.bin` entry 139, name `050000b1`, offset `0x8594a0`, len `0x860`, disc start `0x31870000`;
  - `07000015.bin` entry 102, name `050000b1`, offset `0x609e60`, len `0x860`, disc start `0x33782000`.
- **The 07000014 load path** [M LoaderLoad 0x1d4a00 case 0 builds `"%s%8.8x.bin"` from the
  hashcode with the 0x00F00000 nibble rewritten (type 0/3 → `|0xf00000`); ResetMap_Load
  0x1d6360:31/34 opens type 0 and type 1 with GameState+8]: the front-end `07000048.bin` plus
  every story level before 14 in `mission::LEVELS` order (05 06 07 08 01 02 03 04 09 0a 0b 0c 0d
  11 12 4a 13) plus 14 itself, each with its `07F000xx.bin` anim sibling (all present). **None of
  them carries 0x050000b1.**
- **07000014's own archive** holds `0x0500008c`, `0x05000093`, `0x050000b2` — and not b1 [D].
- **The 11 raw b1 placements** (flags 0x0f, param 0 = 0x050000b1, all kind 7 through the kind
  table): 448, 449, 465, 468, 471, 472, 479, 703, 704, 1029, 1030. At the boot difficulty
  (default 2, 0x1c7e78:25) the sim creates five: **449** (−29.50, −12.46), **468** (−4.99,
  −23.50), **472** (5.87, −21.52), **703** (22.04, −24.98), **704** (−11.87, −20.11) — the
  remaining six sit above the param-2 gate and create no drone.

## 2. Why the pin was wrong: the game's own rewrite [M]

`NDrone2_DefaultInit` 0x14b300:82-95 (decompile `work/decomp/ACTION.ELF/
0014b300_NDrone2_DefaultInit__FP10DIVars_tag.c` lines 82-95):

```c
if (GameState._12_4_ == 0x7000014) {
    if (piVar19[0xd] == 0x50000b1) { piVar19[0xd] = 0x500008c; ... }
}
...
*(int *)(iVar3 + 0xe0) = iVar11;      // the REWRITTEN value goes to Drone+0xe0
switch(iVar11) { ... }                 // the kind switch reads the rewritten value
```

and the render handoff [M 0x14b300:563-565]: `uVar10 = *(undefined4 *)(iVar3 + 0xe0);`
… `AnimObjectNew__FP7obj_tagi(iVar12, uVar10);` — `AnimObjectNew` receives Drone+0xe0, which
holds the rewritten **0x0500008c** (captain or not; a kind-7 captain's swap :536-555 then lands
on 0x050000b2, also own-archive). 0x0500008c is kind 7 — the same row b1 maps to in the kind
table (`0x500008c`/`0x50000b2` cases, decompile lines 247-249) — which is why the kind-level
analysis in the old doc was right while its render conclusion was wrong.

Supporting audit (the game cannot find the raw b1 anywhere else): `AnimProcessSkinData` 0x1bd500
has exactly one caller (`AnimLoadFile` 0x1bdf40:52), `AnimLoadFile` exactly one caller
(`LoaderProcess` 0x1d4880 case 3), and every other `hashtable_additem` site in ACTION.ELF feeds a
different table (emitters, entity params, palettes/textures, scripts) — the skin table's only
writer is the container walk the session gate models. `AnimObjectNew`'s substitution covers only
0x05000090 [M 0x1be910:33-43 full-range constant scan].

## 3. What changed in the port

- `crates/nf-game/src/drone.rs` — `Drone::new` rewrites the param-0 skin
  (`0x050000b1 → 0x0500008c` when `d.level == 0x0700_0014`) before every use (kind, weapon,
  `dr.skin`); `captain_skin` (the draw-side pure form of the +0xe0 rule) applies the same rewrite
  first, so it answers "what is in +0xe0 given the placement" in one place.
- `crates/nf-view/src/scene.rs` — the drone_skins rebuild now runs on the final +0xe0 hash: it
  rebuilds a placement's character instances from the final skin's mains whenever the final hash
  differs from the raw param (the rewritten b1 placements AND the captains — previously captains
  only), keyed into `drone_skins` the same way. A final hash the table lacks still draws nothing
  (the game's `AnimObjectNew` 0x1be910:44-46 miss).
- `crates/nf-view/src/lib.rs` — the G-021 comment block and the carrier diagnostic now state the
  corrected chain (the raw b1 never registers; the placements resolve through the rewrite).
- Docs corrected with the addresses: `docs/systems/drone-skins.md`, `docs/GAPS.md` (G-021 row),
  `docs/models/characters.md` (the 07000014 rows), `docs/systems/missing-models-census.md`.

## 4. The tests (all re-pinned, none weakened)

- `missing_models_census_live.rs` — `07000014.bin → skinless_drones == 0`, message carrying the
  full [M]+[D] chain (the old `== 5` pin contradicted the decompile's rewrite; fixed with the
  address, per the task's rule). All other levels stay `== 0`.
- `drone_skins_all_levels_live.rs` — the 07000014 arm now pins: raw-missing params still exactly
  {0x05000090, 0x050000b1} (the RAW-archive fact [D]); the 0x05000090 substitute entry still
  carries 0x05000093's data; **skinless empty**; every raw-b1 drone's effective skin is the
  rewrite 0x0500008c or a kind-7 captain's 0x050000b2 (`b1_final`), with at least one
  non-captain 8c at the boot difficulty. Header chain corrected.
- `models_characters_live.rs` — the airbase test keeps its two negative b1 assertions (the table
  never holds b1; no character keyed b1) and adds the positive pin: characters built for
  0x0500008c.
- `load_scene_equality.rs` — the 07000014 digest re-pinned for the deliberate scene-content
  change (five placements now draw their characters): `11023d599b9c96e4 7136507 14822037 6558
  6983574 98 13506 173 41 37 32` — +5 characters, +30005 vertices, +59235 indices, +105
  materials, +50005 char_verts, +15 attachments over the old pin. All other rows untouched.
- `nf-formats` untouched (the `drone_setup` rewrite and its unit test were already right — they
  mirror :82-95 for the kind table).

What ran before the last push (all green): `nf-game` 86/86 test binaries ok (546 lib tests);
`nf-view` `drone_skins_all_levels_live`, `missing_models_census_live`, `models_characters_live`,
`load_scene_equality`; `nf-formats` + `nf-audio` full runs ok except the 3 `disc_golden` tests
(the driver's documented slim-ISO allowance). The full four-crate suite in one run did not fit
this sandbox: background processes are killed between tool calls here, so `tools/test/suite.py`
could not run persistently, and a foreground `cargo test -p nf-view` (all binaries,
--test-threads=1) was cut by the 560 s tool timeout mid-run — the four nf-view binaries above
are the ones this task touches and they pass. Next session: `python3 tools/test/suite.py` once
for the total line.

## 5. Live checks for the main agent

Nothing further is required to settle the claim — the decompile plus the disc settle it. One
optional render confirmation (visual, so it belongs to the main agent):
- `--bond -29.5,1.0,-12.5` and `--bond 22.0,1.0,-25.0` on 07000014 (placement 449 / 703): each
  should show a guard character (the 0x0500008c skin's model) where the pre-fix build drew none.
- A fresh-boot gdbwatch on a b1 drone's Drone+0xe0 (address = the drone obj + 0xe0) right after
  `NDrone2_DefaultInit` 0x14b300 returns: expect `0x0500008c` for a non-captain (and 0x050000b2
  for a kind-7 captain), confirming the rewrite on the real hardware path.

## What is left

- One full `tools/test/suite.py` pass on this tree for the single total line (the four crates'
  changes are each covered by the runs listed above).
- The "What is not traced here" tail of `docs/systems/drone-skins.md` can drop its first bullet
  (the weapon now derives from the rewritten skin, matching the game's +0xe0 order) — left for
  the next message to keep this one scoped.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the
session (checked the CLI and the IM gateway metadata; nothing numeric to copy).
