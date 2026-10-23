# The never-mentioned animation queries, the anim-cache stubs and the debug spawners

The whole-ELF scan (2026-10-20) found 8 animation
functions whose name or address appears nowhere in `crates/*/src`. This file classifies each; the
citations live in the Rust code so the scan sees them. Evidence tags: [M] decompiled code
(`work/decomp/ACTION.ELF/00<addr>_<name>.c`) or my own disassembly reads, [D] disc, [L] live
PCSX2 measurement, [S] suggested.

The reference sweep behind every verdict: every `JAL` instruction word re-derived for the target
address across the loaded `.text`/`.vutext`, every raw address word in `.data`/`.rodata`/`.sdata`,
every `lui`+`addiu` address-formation pair, and the decompile call graph. The functions' own
`.symtab` entries are the only other hits. The sweep is the "evidence basis" block in
`docs/port-map/unreached.md` §863.

## The two reached flag queries — ported as predicates

### `AnimObjectIsAiming` 0x1c6900 — ported: `nf_game::anim::is_aiming`
`obj != 0 && obj+0xdc (anim object) != 0 && (animobj+0xc8 & 0x180) != 0` [M 001c6900]. Callers [M
decompile + JAL scan]: `NDrone2_DoTracking` 0x1479d8 (4 sites — its `AnimObjectSetAiming` stop path
when the flag is already up, and the +0x74 tracker check) — the drone AI tick, reached on every level
with aiming guards. The port folds the 0x180 flag into the aim/fire state list
(`drone.rs` `aims_torso`, the documented stand-in) and never carried the flag word; the predicate
itself is now `anim::is_aiming(flags)` with the exact 0x180 mask, and `aims_torso`'s doc cites the
game function. Tests: the mask semantics and the null-object arm (`anim.rs`, sister = the head-track
bits are not aim bits).

### `AnimObjectIsHeadTracking` 0x1c6850 — ported: `nf_game::anim::is_head_tracking`
`obj != 0 && obj+0xdc != 0 && (animobj+0xc8 & 0x30) != 0` [M 001c6850]. Callers [M]: `NDrone2_DoHeadTracking`
0x1472c8 (2 sites — the early return and the re-decide). The port's fold is the `tracking` field +
`head_yaw`/`head_pitch` machinery (`drone.rs` 0x1479d8/0x1472c8 fold, cited there); the predicate is
`anim::is_head_tracking(flags)` with the 0x30 mask, same test pair.

## The two empty cache stubs — cited at the load path

### `AnimCacheInit` 0x1c6390 — empty; cited at `nf_view::load`
Body `jr ra; nop` [M disasm]. Caller: `ResetMap_GameInit` 0x1d5080:37, on every level start [M].
There is no anim cache in the shipped build to init (the whole family 0x1c6390/0x1c6398 Enable/
0x1c63a0 Disable is empty — 0x1c6398/0x1c63a0 already covered by `anim-script-rest.md`).
Cited at the port's level load (`nf-view/src/lib.rs`, next to the existing `AnimPreLoadInit`
0x1c63b8 note): the fresh per-load maps ARE the reset.

### `AnimSeqGetHdrBuf` 0x1c7998 — returns 0, result unread; cited at `nf_view::load`
Body `jr ra; or v0,zero,zero` [M disasm] — always returns 0. Caller: `ResetMap_Load` 0x1d6360:135
(`jal` at 0x1d67c4, delay slot `li a0,1`) [M]; the next instructions immediately overwrite the
argument registers for the following `LoaderLoad` — v0 is never read [M disasm 0x1d67c4-0x1d67d8].
A retired header-buffer hook. Cited at the port's load path next to the `LoaderLoadAnims` block.

## The debug family — never reached (or never acts) in the shipped build

### `AnimDebugHandler` 0x1bc6e8 — unreached; recorded in `unreached.md`
2240-byte debug anim-viewer state machine: watches the `AnimDebugDisplay` global 0x30cb5f [M
symtab], and per its state spawns mannequin objects (poses 0x6000243/0x6000259/0x600041a/0x60002bd+
0x60002ed aim-blend/0x60006e7/0x600055c+0x600063b facial) around a found object, resetting their
+0x30 position from `origpos` 0x319c60 when the script's +0x78 word carries 0x40000000 [M 001bc6e8].
ZERO references in the loaded image: no `JAL` (verified over the whole `.text`/`.vutext`), no
address word in any loaded data section, no `lui 0x1b`+`addiu 0xc6e8` pair, no decompile caller
[M]. The linker kept it; the shipped game cannot call it.

### `AnimCacheDump` 0x1c63b0 — unreached; recorded in `unreached.md`
Body `jr ra; nop` [M disasm]. Only `JAL` is at 0x1bc6fc — inside `AnimDebugHandler` (above) [M];
no data references. Dead even in the debug path (the dump was emptied).

### `AnimDebugCreate` 0x1bcfa8 — reached as a call, never acts; recorded in `unreached.md`
`JAL`ed unconditionally from `Player_Init` 0x1910f8:117, right after the player's `AnimObjectNew`
[M]. The body gates on `GameState+0xc`: 0x07000043 / 0x07000049 → spawn one object per loaded
0x5000000-family model hash in a circle of radius `count * 0.2387324` m (1.5 m of circumference per
model) at `0.09 - AnimObjectInitialPose(...)` height with yaw `atan2f(-sin, -cos)` [M 001bcfa8:72-138];
0x07000041 → one object 3.3 m behind-ish the player (yaw − 0.3) with anims 0x600011b +
0x6000128 facial and `BodyGlow_Create` [M 001bcfa8:140-160]. Those three codes are wrapper codes
the retail game never loads: every 0xE8 placement param-0 on the disc scanned — no loader of
0x07000041/43/46/49, and the decompile has no loader constant for them [D, pinned in
`doublecross_transitions_live.rs:44`]; 0x07000041 is not in the MP sound-bank family either
(`Sound_Ready` gives it SP bank 0xb [M 0x1dceb8:91; port `sound.rs:192`]). So on every shipped
on-foot level, front end and MP, the call returns at once. The port's player-init site
(`scene.rs`, the `Bond::stand_at` spawn) cites it; the body's flow is deliberately NOT ported
(the mannequin circle needs the game's 0x5000000 model registry, which nothing in the port
maintains, and no shipped level can ever show it).

### `AnimObjectInitialPose` 0x1c64e0 — unreached; recorded in `unreached.md`
Adds a temp script (0x600035e, or 0x6000259 when animobj+0xc8 has 0x400), forces its frame pointer
to 0, clears the 0x80000000 loop bit, ticks `AnimObjectUpdate`, reads bone 0x80000037's world
position via `AnimGetBoneWorldTrans`, deletes the script and returns that Y [M 001c64e0 — the float
return is `fVar11 - AnimObjectInitialPose(obj)` at the caller, i.e. the root-bone height above 0].
Its only `JAL` is inside `AnimDebugCreate`'s never-acting body [M]; no data references, no other
callers. anim-script-rest.md already lists it among `AnimScriptDelete`'s callers [M
`anim-script-rest.md`]; nothing reaches it.

## Scan / calltree

`tools/calltree/calltree.py` on the two ported predicates: both are leaves over the caller's flag
word — no callees, so no never-mentioned callees. The six cited/unreached functions port no new
call edges.
