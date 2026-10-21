# The animation-script helpers the AI reaches (call-tree gap, task 850 `anim-script-rest`)

The call-tree check (`tools/calltree/calltree.py --all-mentioned-dstates --list`, master 20874c31) walks the
game's code from every DSTATE handler the port mentions. This task owns the 19 animation-script functions the
walk reaches but the port never named. Every one is now classified here and cited at the Rust that does (or
deliberately does not do) the work — none is silently skipped. Citations live in the code; this file is the map.

Evidence tags: [M] decompiled code (file `work/decomp/ACTION.ELF/00<addr>_<name>__*.c`), [D] disc, [L] EE/live
measurement, [S] suggested/unverified. "Mentioned" for the checker = the address or the function name appears in
non-test `crates/*/src`; the checker's own doc notes mentioned ≠ ported-complete, so each entry below states
which it is.

## The three empty/trivial ones

### `AnimCacheDisable` 0x1c63a0 — empty [M]; cited at the caller
Body is `return` [M 001c63a0]. Only caller: `LoadableLoad` 0x1d69e8 (first call [M 001d69e8:10]). The port's
`LoadableLoad` pass is the loadable-pack stream in `nf-view/src/lib.rs` (`loadable pack` loop); the citation
there notes the function is empty so no cache gate is modelled.

### `AnimCacheEnable` 0x1c6398 — empty [M]; cited at the caller
Body is `return` [M 001c6398]. Only caller: `LoadableReload` 0x1d6ab0 (before its own 0x4000000 flush [M
001d6ab0:11]). Same citation site (`nf-view/src/lib.rs` pack stream, covering both).

### `AnimDistanceTableInit` 0x1c76e8 — one store; cited at the lazy equivalent
Sets the global `nDistanceTables = 0` [M 001c76e8]. Only caller: `AnimPreLoadInit` 0x1c63b8. The port has no
global table registry — tables are built lazily per sequence (`nf-game/src/anim.rs` `DistanceTable::build`,
next to the cited `AnimDistanceTableCreate` 0x1c4060 / `AnimDistanceTableGet` 0x1c76f0); the doc line at that
struct records the fold.

## The level-load reset pair

### `AnimPreLoadInit` 0x1c63b8 — fresh-per-load reset; cited at the load
With param 0 [M 001c63b8]: `AnimTagTimeStamp = 0x80000001`, `AnimScriptTimeStamp = 1`, the skin-scale globals
(Glb_InvSkinScale / Glb_SkinScale / DAT_00319d84/88/94/98) = 1.0f, 0x200 bytes at `pSkeletons+0x1fc` zeroed,
`AnimDistanceTableInit` called; always `Glb_AnimPostLoadDone = 0` and 7 u32s at DAT_002c7248 zeroed. Callers:
`ResetMap_GameInit` 0x1d5080 and `LoadableReload` 0x1d6ab0. The port rebuilds the skeleton/sequence/script
maps on every load (`nf-view/src/lib.rs` `load_impl`): nothing persists across loads, so the fresh maps ARE
the reset — cited there. No port counterpart exists for the globals themselves (no persistent anim state).

## The seq-data pipeline (load → play → free)

### `AnimProcessSeqData` 0x1c4ce0 — the same stream walk; cited at `Sequence::parse`
For a 0x4xxxxxx id [M 001c4ce0]: hashtable lookup, walk of the compressed frame stream (start +0x40, +0x38
frames / +0x3a channels, the `u16 >> 6` cumulative count, 4-byte skip per set low-6-bit coefficient bit) to
find the stream end, stores end pointer/byte count into the entry, and `*(entry+0xc) &= 0xf` then `|= 7` for a
hardcoded list of 0x0400xxxx ids. Caller: `AnimLoadFile` 0x1bdf40. The port's `Sequence::parse`
(`nf-formats/src/anim.rs`) walks the SAME bytes field-for-field — but parses the coefficients instead of
skipping them, bounds channels by their header byte length instead of storing an end pointer, and does not
apply the `|= 7` flag side effects (nothing on the disc's data needs bits 0-2: the port evaluates every
sequence through the same polynomial path and the live suite passes). Cited, deviations listed.

### `AnimSeqGetInfo` 0x1c79a0 — the field copy; cited at `Sequence::parse`
Copies the raw record's fields into the runtime sAnimSeq_tag [M 001c79a0]: hash → +0x94, +0x10..+0x34 →
+0x70..+0x7c/+0x50..+0x68, +0x38/+0x3a/+0x3c/+0x3e → +0xa6/+0xaa/+0xa9/+0xa8, low nibble of +0xc → seq+0x98
bits. The port's `Sequence` struct fields ARE those values, read once at parse (same doc site, cited).

### `AnimSeqDelete` 0x1c72e0 — ownership; cited at the seq map
Frees seq+0x88, seq+0x84, then the seq [M 001c72e0]. The port's `Sequence` owns its decoded channels and
drops with the level's maps (`nf-view/src/lib.rs`, cited at the kind-4 parse loop).

### `AnimScriptNew` 0x1c1398 — node creation; cited at both node equivalents
Allocates the 0xc0 sAnimScript_tag, memsets, writes the `0x159f5ab2` canary at +0x40, creates the +0x50 body
seq (`AnimSeqNew(len<<5, 0x2404)`), the +0x54 facial seq when the skeleton's +0x2e byte is set, links into the
object's +0x44/+0x48 list [M 001c1398]. Callers: `AnimScriptAdd` 0x1c6d40 / `Append` 0x1c7000 /
`AppendBlend` 0x1c17c0 / `AddSpeed` / `AddReverse`. The port's node equivalents: the AI side `Outgoing` push
in `drone.rs` `begin_blend` (the `SetHTAnim` switch Add vs AppendBlend) and the MP side `MpBodyAnim::append`
(`nf-game/src/mp/body_anim.rs`) — both cited with the structural deviations (Vec+stamp instead of the linked
list, hash-resolved seq slots instead of `AnimSeqNew` buffers, no canary/malloc-failure path).

### `AnimScriptDelete` 0x1c14f0 — the faded-out delete; cited at `resolve`
If deleting `Glb_pCurrentScript` sets `Glb_bScriptDeleted = 1`; fires the +0x68 OnStop callback; clears the
+0x64 backref; deletes the +0x50/+0x54/+0x58 seqs (`AnimSeqDelete`); unlinks +0x44/+0x48; writes the
`0x983c4837` freed marker; frees [M 001c14f0]. Callers: `AnimSetUpdate` 0x1c0938, `AnimScriptTick` 0x1c2630,
`AnimFrameResolve` 0x1c35f8, `AnimSetDeleteAll` 0x1c6928, `AnimListDelete` 0x1c6b98, `AnimScriptAdd*`,
`BOTSTATE_stopAnim` 0x129918, `AnimObjectInitialPose` 0x1c64e0. The port's delete = the faded-out
`layers.retain` in `MpBodyAnim::resolve` (cited) and the drone side's `outgoing.retain_mut` after
`Outgoing::tick`. The OnStop callback / backref clear are not ported — the shipped game's only OnStop setter
call is `DroneAnim_SetHTAnim` 0x13b1c8 and it writes NULL [M], so no callback body ever runs through the
modelled scripts; the reentrancy flag is structurally unreachable in the port.

### `AnimScriptCallOnEvent` 0x1c71f8 / `AnimScriptCallOnStop` 0x1c7200 — the callback slots; cited inlined
Store a function pointer at script+0x70 / +0x6c [M 001c71f8 / 001c7200]. Sole shipped caller of both:
`DroneAnim_SetHTAnim` 0x13b1c8, which writes NULL [M]. The port inlines the +0x70 dispatch as the op-4 arm of
`script_commands` feeding `event_func` (`drone.rs`, cited); the +0x6c stop slot is documented at `anim::Outgoing`.

### `AnimScriptSetAnimRotation` 0x1c7208 — the anim-rotation flag; cited at `begin_blend`
Param 0: clear +0x78 bit 0x20000; else +0xa0 = 8.0f and set the bit [M 001c7208]. The bit's effect — the
fading script's root yaw belongs to the object — is the port's `Outgoing::unyaw`, taken in `begin_blend`'s
`old_turn` arm (the yaw `SetHTAnim` moves to +0x54C for `SnapRotate`) and applied in `local_pose` (cited at
`drone.rs` `begin_blend`; the +0xa0 time-scale field and the param-0 clear arm have no port path).

## The getters and feeds

### `AnimGetFrameTransRaw` 0x1c73d0 — the raw frame translation; cited at `RootTrack`
`AnimSeqGetInfo` + `AnimFrameCopy(frame, out, seqinfo, 0)`, VECTOR out [M 001c73d0]. Caller:
`Player_CollWire` 0x197130 (the wire grab's hand-reach correction [M 0x1974b8-0x197538]). The port's
`wire.rs` `RootTrack::of`/`at` sample channels 0/1/2 per frame through `Sequence::value` (the
`AnimFrameCopy` 0x1c5668 evaluator); consumed by `Bond::grab_wire`. VECTOR[3] is unused by the caller.
Cited at `wire.rs`.

### `AnimGetNDiscreteObj` 0x1c66b8 — the discrete-object getter; cited at `discrete_parts`
Returns obj->anim(+0xdc)->discrete[i] from the +0xc0 array, bounds-checked against the skeleton's discrete
count (+0x94+0x2d), 0 on failure [M 001c66b8]. Caller: `Script_GetInterp` 0x1da158. The port computes the
skin's rigid parts on demand at the script's frame (`drone.rs` `Data::discrete_parts` / `discrete0`, consumed
via `World::discrete0` in `nis.rs`); the bone `.get()` filter is the bounds check, an empty vec the failure.
Cited at `drone.rs`.

### `AnimObjectDelete` 0x1bee80 — the anim teardown; cited at the kill site
`AnimSetDeleteAll` + `AnimListBuild(1,0,0,1)` + `AnimListDelete`, a `control_delete_object` per non-null
discrete entry at anim+0xc0, frees the five track arrays +0xac..+0xbc and obj+0xdc [M 001bee80]. Callers:
`control_delete_object` 0x133178, `Player_SetWeaponAnim` 0x1a9338. The port's `Obj::Anim` is a value dropped
by `objs.list[o] = None` (`nis.rs` `Script::kill`, cited); the discrete parts are rebuilt per tick
(`script_parts`), so there is nothing to free.

### `AnimObjectSetHeadTracking` 0x1c6748 — the head-track state write; cited at the head-track block
Writes target → animobj+0x70, range² → +0xE8; target null: clear +0xC8 bit 0x10 and zero +0x70; else set 0x10,
mode adds 0x40 (+0x50 with param 6, which also stores the bone byte +0x104), target stored, re-look delay
`Rand(FR/4)+FR/10` → +0x106 when mode 1 [M 001c6748]. Callers: `NDrone2_DoHeadTracking` 0x1472c8,
`NDrone2_HeadTrackObj` 0x14ada0, `Drone_Delete` 0x136ce8 (clear form). The port's head-track block
(`drone.rs`, the `tracking`/`track_recheck` fields) is cited there: `tracking` = the 0x10 flag,
`track_recheck` = the re-decide timer, 3.0² = the +0xE8 range², clear = the `tracking = false` arms.
Not modelled (stated in the citation): the +0x106 random snap-delay draw, the 0x40 target-is-object bits —
`HeadTrackObj` 0x14ada0's arbitrary-target form is unported (that task owns it; the mention here is an honest
"unported" note, not an equivalence claim).

### `AnimTagGetUnique` 0x1c77c0 — the stamp counter; cited at `next_stamp`
Returns `AnimTagTimeStamp` then increments it, skipping 0 [M 001c77c0]; the level reset seeds 0x80000001
[M 001c63b8]. Caller: `AnimSetAppend` 0x1c07b8. The port's `MpBodyAnim::next_stamp` (starts 1, increments,
never 0) is that counter — the `Layer.stamp` field annotated `+0x84` — cited at the increment site.

## The facial-speech pair — caller branch NOT ported (documented gap, not claimed equivalent)

61 of 346 `Drone_SfxInfo` entries carry a facial anim id [L, docs/systems/speech.md §1.2], so on levels that
load facial scripts this branch is live. On the port's tested level 07000005 no facial scripts load: the
game's own `hashtable_getitem` miss clears `e.facialAnimId` to 0 (permanent) and the line falls through to the
direct play [M 0x141b00, speech.md §1.3]. The port's `Drones::speech` skips `l.facial != 0` lines entirely —
conservative, and the whole branch is documented as unported there [S standing note].

### `AnimScriptPeekSfx` 0x1c6ca8 — the walk exists; the caller's use is unported
Walks a seq's command stream (count = +6 & 0x1ff, per-command advance (n+3)/(n+2)+1 u16s), returns the first
op-1's sfx id, aborts on op > 4 [M 001c6ca8]. The port's `Script::parse` (`nf-formats/src/anim.rs`) does the
same walk and keeps every op-1 (`args = [frame, sfx]`) — cited there as the superset; the drone plays them by
frame window (`script_commands` op-1 arm). What is NOT ported: the game caller's branch —
`NDrone2_SpeechSFX` 0x141b00's facial path (PeekSfx → `SFXPreCache` + the T+20 pending append), documented at
the speech skip site and in speech.md §1.3-1.4.

### `AnimFacialAppend` 0x1c6490 — the consumer exists; the append is unported
`AnimScriptAppend(obj, id)`, result+0xb5 = 3 (stop at end), result+0x80 = 0xe0000002 (the talk marker → facial
slot 2), animobj+0x109 = 2 (speaking) [M 001c6490]. Callers: `NDrone2_SpeechSFX` 0x141b00 (both FACIAL
sub-branches end here — at once when the script has no op-1 sfx, at the +0x48 timer via
`NDrone2_ControlSTANDARD` 0x1490f0 §1.4 when it has), `AnimDebugHandler`/`Create` (debug only). The port's
slot-2 consumer exists — `facial_blend(out, slot0, slot1, slot2, …)` (`nf-game/src/anim.rs`, cited) — but the
scene passes `slot2 = None` and nothing appends. Porting it needs the speech-side talk slot plus the pending
timer state (+0x4c/+0x37/+0x48) and is queued, not smuggled in here.

## Live checks for the main agent

None required for what landed (comment/doc-only; no behaviour change). If the facial port is queued, the
useful spot-checks on a level WITH facial scripts loaded would be: watch `Drone_SfxInfo[idx]+4/+8` and drone
bytes +0x35/+0x36/+0x37 / +0x53c while a guard with a facial line talks (`tools/trace/gdbwatch.py`, expect the
+0x37=1 → T+21 append → +0x36=1 sequence of speech.md §1.3-1.4).

## What a future facial port needs (not done here)

`Drones::speech`: for `l.facial != 0` with the facial script loaded on the level, peek the parsed commands for
the first op-1 (`find_map(|c| c.op == 1)`) — sfx found → schedule (precache + play at T+20, pending state
bytes) ; none → append into a new drone talk slot consumed by `Scene::facial_weights_at`'s `facial_blend`
`slot2` (the plumbing exists). Real-path tests: a talk request on a level whose speech line carries a loaded
facial script asserts the slot set / the sfx scheduled; sister: the same line with the script absent plays
direct. Untested on 07000005 (no facial scripts load there [M/speech.md]), which is why it is not in this change.
