# The driving load path — files, missions, and the world assembly (DRIVING.ELF)

Status: first pass. Evidence: [M] IDA database of `work/elfs/DRIVING.ELF` (function + address;
strings quoted with their addresses), [D] disc, [S] suggested. This doc is also the **carve
manifest's driving half**: the file set the engine touches (the on-foot half is FILES.BIN +
`PS2/ENGLISH/SB_*` + `PS2/MUSIC/MFX_*` + `STREAMS` + movies, per docs/PLAN.md).

## 1. The file layout (the engine's own strings) [M]

| path | role | evidence |
|---|---|---|
| `cd:driving\misc.viv` / `driving\misc.viv` | the shared archive, loaded unconditionally ("Opening misc.viv", "*** misc.viv found, and being used! ***") | 0x375a78-0x375ac0 |
| `/DRIVING/*.VIV` [D disc manifest] | `MIS01/MIS3/MIS4/MIS11/MIS13A/MIS13B/MIS13C` (the seven mission names), `MISC` (189 MB shared), `RACE` — the eight archive stems; each mission VIV pairs with `MISxx.MUS` (music) and `MISxxEN.SPE` (speech) | [D LBA table, `nf-extract manifest`] |
| `/DRIVING/DRIVING.SYM` [D] | ships on the disc but does **not** describe this ELF (4 % boundary rate, `_end` 0x55cf04 vs 0x4279b4 — symfile.rs header, WORKLOG 0001): naming hints only | [D 559488, sha1 8dbd38f9…] |
| `%s.viv` / `%s\%s.viv` | the per-mission archive (MIS01.VIV-class), opened through `sub_2D6ED8` [M 0x2d6f30] | 0x376830/0x3c1a78 |
| `data\actors\skeleton\0skel.o` … `8skel.o` | the nine skeleton objects | 0x326168-0x326438 |
| `data\actors\anims\ALookup.bin`, `BkDflt.txt` | anim lookup + defaults | 0x363fb8/0x363ff8 |
| `data\actors\models\`, `data\actors\textures\`, `data\actors\weapons\` | actor asset prefixes | 0x364b08-0x367400 |
| `data\Render\Actors.ssh`, `data\render\ext.ssh` | the two SSH texture banks | 0x366b40/0x37f1d0 |
| `data\car\model\` (×5 sites) | per-car CRP models | 0x36b110-0x37a470 |
| `data\tuning\` (+ `\default.tun` fallback) | the .tun namespace (drive-physics.md §2) | 0x36da58-0x36da68 |
| `data\sim\attrib\` | the `.atr` attribute tree (`pvehicle\<car>.atr`) | 0x374af8 |
| `data\loading\<name>%s%s` / `<name>.ini` / `default.ini` | loading screens (below) | 0x36fad8-0x3757b0 |
| `data\track\` | the road/track data — read by `sub_174198` [M 0x17492c] | 0x375b78 |
| `data\audio\` | the driving audio banks | 0x376ca0 |

## 2. Mission identity is a file probe, not a code map [M]

`sub_173EE8` (0x173ee8, "Show loading screen") resolves the mission by probing
`data\loading\<name>.ini` for existence (`sub_2A9508(0x1004D4, name)` — the first probe is
`"paris_mis01"`, then `snow1a_mis3`, `snow2a_mis4`, `uw_mis11`, `junglea_mis13a`,
`jungleb_mis13b`, `junglec_mis13c`; **seven names, five missions** — mis13 has three maps).
The match builds `<name><suffix><ext>` (suffix `"W"` when `*(0x333760+92) != 0` [M 0x173f14 —
a wide/progressive-screen flag, `sub_2A9098` is `sprintf`]) for the `.ssh` loading screen plus
the `.ini` (jungleb/c fall back to `default.ini` [M 0x174150]).

Port consequence: the mission runner resolves its assets the same way — a probe chain against
the carved pack — no hardcoded 0x09xxxxxx→name table exists in the engine [M-negative: the
probes ARE the map]. The front end's 0x09xxxxxx codes (menu.rs) must map to these names in the
port's own layer [S — the mapping table is still to be measured: the caller of this picker and
its siblings].

## 3. The boot chain and the low-memory name mailbox [M]

- **The open wrapper** `sub_175A28` (0x175a28): scans the current mission-name string for the
  **last underscore** and opens `driving\<after-underscore>.viv` — `paris_mis01` →
  `driving\mis01.viv` [M 0x175a50-0x175a68: the `0x5F` scan; 0x175a94:
  `sub_2D6ED8("driving", v2, dword_32E7C0)`].
- **The name mailbox**: the string lives at `0x1004D4` — *below* this ELF's load base
  (0x107000), i.e. the low-memory hand-off area. PLAN.md's suggested PTP copy
  (`0x0031CF90 → 0x100000` on the on-foot side) puts the mission name at **PTP+0x4D4**; the
  DRIVING side reads it from the fixed low address [M 0x175a44/0x173f10-era probes all pass
  1049844]. The hand-off field is now measured from the consumer side; the on-foot writer
  (which fills it before the launch) is an ACTION.ELF-side check [open].
- **The VIV open** `sub_2D6ED8` (0x2d6ed8): clears the name buffer (`0x427398`, 256 bytes),
  the loaded flag (`0x355C88`) and the mode (`0x355C90` = arg 3, `dword_32E7C0` at the call
  site); `sprintf("%s\\%s.viv")`; opens through the file layer (`sub_254308`, handle →
  `0x355C8C`) when it exists (`sub_256EE0`) or a host-mode override (`0x32E68C`) is set [M
  0x2d6f04-0x2d6f7c].
- **The close** `sub_2D7078` (0x2d7078): closes the handle when the flag is set, clears it [M
  0x2d7094-0x2d709c].
- **The streaming sidecars** `sub_2D6FA0` (0x2d6fa0): when mode == 1, walks the streaming
  system's request/loaded lists and WRITES `<dir>\<name>_req.txt` / `_in.txt` (one entry name
  per line) through the file layer [M 0x2d6fe0-0x2d7024; the writer `sub_2D6DB8` walks a
  linked list — nodes: next at +0, name at +8 — sprintf "%s\n", write]. Diagnostic dumps of
  the requested/loaded entry sets, not inputs.

## 3.5 The boot entry — `sub_174198` (0x174198) is `main`, not a track reader [M]

Corrects the load-path first pass's guess: the `data\track\` string's consumer is the boot
function. It initialises the IOP (SIO2MAN, MTAPMAN, PADMAN, USBD, MCMAN, MCSERV, LIBSD, SDRDRV,
SNDDRV, SNPROFIL [M 0x1741f8-0x174258]) and reads the **low-memory mailboxes** of the PTP page
(the 0x100000 hand-off region): `0x1004B4` = the base-ELF path (`cdrom0:\BASE.ELF;1` — the
on-foot ACTION ELF; DRIVING can chain back to it [M 0x174284-0x1742a4]), `0x1004D4` = the
mission name (§3), `0x100540` = the localization `.loc` (`IDL_BRT.loc`/`IDL_FR.loc`… per the
mode at `0x100988` [M 0x17435c-0x1743e8]). The track-data consumer itself is still unpinned —
removed from the open list only when found [open].

## 3.6 The world-reset orchestrator — `sub_176658` (0x176658) [M]

The mission/world load sequence (276 decompile lines; every stage labelled with the engine's
own `printf`):

1. "Reset Ray Shells" `sub_199780`, "Reset Sim" `sub_1FEDA0` (the physics sim reset —
   drive-physics.md §1's timestep writer), "Init Clock" `sub_1799D8`, "Init Events"
   `sub_1755F0`, "Init Scheduler" `sub_179C48` [M 0x176694-0x1766ec].
2. The schema registrations ride here: `sub_1968F8` (pvehicle) + `sub_185668`
   (Physics:Physical/Friction) [M 0x1766f4-0x176708] — the swap-slot global `dword_3D0D10`
   (drive-suspension.md §1) points into blocks fed by these.
3. "Init Game Render" `sub_1C4008`, "Init Sound" `sub_137E28` [M 0x176710/0x176744].
4. **The mission-name probes and the speech stem** [M 0x17680c-0x1769cc]: the chain probes the
   name mailbox for `paris_mis01`/`paris_map`, `snow1a_mis3`/`snow1a`, `snow2a_mis4`/`snow2a`
   (+`snow2a_race`), `uw_mis11`, `junglea/b/c_mis13x` — the second names are alternative
   mission variants — and builds the **speech-file stem** (`mis01`/`mis3`/`mis4`/`mis11`/
   `mis13a`/`mis13b`/`mis13c`) via `sub_177260` (0x177260 = GCC `std::string::replace` — the
   `bastring.cc` asserts identify it), used as `driving\<stem><locale>.spe` with English
   fallback [M 0x1769ec-0x176a74; matches the disc's `MIS01EN.SPE` manifest [D]].
   ~~The earlier draft read this as a separate "world database" open — wrong:~~
5. **"Init World" is the track `.crp` init** [M 0x176ba4-0x176be4]: `sub_22FAF8(dword_33AAC8)`
   initialises the track object whose `+0x14` (the same pointer Init RoadNetwork reads) is the
   track `.crp`; the AI-elements init then locates the **`Map `** article
   (`sub_2D51F0(crp, 0x4D617020)` — tags are byte-reversed ASCII) and its **`AIEl`** record
   (`sub_2D5A10(·, 0x4149456C)`) and hands it to `sub_12BE48` [M 0x176bb8-0x176be4] —
   exactly crp.md's head list (Arti×N + CDat + Map + RNgp + Shar). There is **no separate
   world database**: the static city IS the track `.crp`'s `Arti` articles, rendered through
   the embedded module's GeoPrim models.
6. Volumes ride the PTP page too: effects `0x100964`, music `0x100968` (÷100 into the audio
   statics), audio mode `0x100A28` [M 0x176ae0-0x176b58]; audio init opens `data\audio\`
   [M 0x176ae0].
7. **"Preload common models" = the player's car** [M 0x176cac-0x176d08]: the `.atr` attribute
   `rendername` of the car record names `data\car\model\<rendername>.crp` (loaded via
   `sub_1C7F48`), with `SECONDARY_TYPE` as the secondary — the car-model load path.

## 5. The render-model chain is parameterized primitives, not baked vertices [M+D, 2026-11-19]

The probe (`crates/nf-formats/examples/crp_module_dump.rs`, dev tool) followed the chain from a
`__Model::<article>::Model[0].NN` symbol slot: symbol (4-byte pointer) → descriptor
(`0x3439A8`-class: pool pointer at +0x00, count 2 at +0x08, an identity f32 matrix +0x0C..0x98,
pointers at +0xA0/+0xB4/+0xB8) → an **`rvrl` pool head** (tag `0x7276726c`, sub-tag `prp`,
capacity 49, links pointing at itself = EMPTY in the file — the pools populate at load). The
namer pair `sub_2CD8C0`/`sub_2DC1B0` are debug-name lookups, not consumers.

Reading: **GeoPrim = geometry primitive** — the road tiles are parameterized primitives
(the ±1 unit descriptors + counts, never baked ±10-20 vertex arrays), generated by the shared
render module `data\render\eaglrm.o` — a real ELF object inside MISC.VIV [D census]. The vertex
unlock is therefore: extract `eaglrm.o` from MISC.VIV, disassemble its GeoPrim generator
(mips-linux-gnu-objdump at /opt/cross/bin works on the object), decode the parameter layout the
`rvrl`/descriptor records feed it. `Wmap`/`iN` give the placements.
- The boot flow: which of the above runs in which order per mission start (the dispatcher
  around `FUN_00159d10`'s world list), and the PTP hand-off entry.
- The audio bank names under `data\audio\` (ASF/EA — the port has none of this yet).
- The carve manifest assembly: this section's table + the per-mission `.ini`/`.ssh` probes and
  the VIV entry lists (each MISxx.VIV's contents, dumped by `nf-extract bigf-extract` on the
  ISO — the VIVs live in the image's `/DRIVING/` directory [D]).
