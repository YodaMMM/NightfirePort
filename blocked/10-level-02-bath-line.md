# 07000002: the servant's line in the bath scene

In the bath scene (about frame 4882) the servant's spoken line may stop before it ends. A fix for the same symptom in the opening scene landed (starting a voice now clears a stale end flag). Run the app path with NF_SILENT=1 NF_NULL_MIXER=1 NF_SOUND_LOG=1 (the real mixer at real time, no audio device) through to the bath scene, read the line's start/stop and played samples against its disc length, and fix any remaining cause. Headless tests run the sim ahead of the mixer and starve streams; do not use them as evidence for stream length.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-02-bath-line`, report in docs/reports/level-02-bath-line.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-08T21:23Z

---
## Report (blocked, 2026-11-08T22:10Z)

# level-02-bath-line — the servant's line in the bath scene (07000002)

Status: **investigation landed, the fix is NOT landed.** The sandbox expired mid-task; the next
message continues from `s/level-02-bath-line` (WORKLOG has the full trail).

## What the bath scene's lines are

The bath scene is the mid-level cut-scene **SP #399, script 0x0600007f** (pack 07900002), fired by
ch 0x35 = SEQ_AND #496 of (0x3B = AND #397 of the four bathhouse-guard death channels + the rescue
0x49, 0x3C = the bathhouse Touch #495) [D placements/triggers, docs/levels/07000002.md]. Script
length 740 ticks [D nf-extract nis].

Its **spoken lines ride the two Anim streams' op-1 sfx**, not the NIS op-0x16 Sounds:

| speaker | anim script | skin | op-1 (frame, sfx) | resolves to |
|---|---|---|---|---|
| stream 1 | 0x06000502 | 0x05000034 | (2, **0x250**) | slot-11 bank effect 0x250 → **STREAMS.LUT[285]**, type-9 end marker pos 44651 → 44 544 ADPCM bytes → 77 952 samples = **3.535 s** @22050 |
| stream 2 | 0x06000501 | 0x05000009 | (0x11e, **0x253**) | slot-11 bank effect 0x253 → **STREAMS.LUT[284]**, end pos 48975 → 48 896 bytes → 85 568 samples = **3.881 s** |

[D all of the above: the level bank (SBINFO slot 11, Sound_Ready 0x1dceb8 case 0x7000002 → bank 2)
carries both effects, each one sub-track with FileRef < 0; the LUT headers' markers read directly.]

The script's own op-0x16 Sounds (418 = 0x1a2, 508 = 0x1fc) are short water barks, and 0x1a2 is
**not in the level bank at all** (only spare slot 5, STREAM LUT[65] 0.27 s) — the game's one-bank
load never plays it; the port's spare-bank extension does. Not the subject.

## What the repro showed (app sound path, real route)

`crates/nf-view/tests/bath_line_repro.rs` (committed): the walkreal route spawn → garden → west
wing → the geisha-1 fight (real fire, no teleports) → 0x3B, then the walker's headless sound is
swapped for the app's `build_sound` output under `NF_SILENT=1 NF_NULL_MIXER=1 NF_SOUND_LOG=1` (the
null mixer = the device path's mixer and voice timing at wall clock), and the walk into the touch
zone plus the whole scene run **paced at 1/60 s per tick** (the app's per-frame order: step, then
`sound_tick`).

- **Line 0x250 played to its full disc length.** Key-on f18945 (scene tick 1) via the cut-scene
  precache takeover (`held true` + "takeover … keys on 1 voice(s)" in one batch — the anim's
  case-2 emits PreCache and the AnimSound on the same tick [M 0x1c1b38 case 1]); end flag at
  f19153 = 208 updates later; kill "sample-ended"; the mixer saw the voice end at **77952/77952**.
  The stale-end-flag fix of 74ee9b749 holds on this path.
- **Line 0x253 never reached the driver at all.** Across the whole paced scene there is no
  PreCache, no Start, no kill for 0x253 — while both anim scripts sit in `sim.scripts` with their
  op-1s intact (06000501 op1 [286, 595] [L port runtime]). The divergence is **upstream of the
  audio driver**: stream 2's `Script_Interp` case 2 (the Obj::Anim tick) never emitted the op-1
  requests. The frame-4882 line in the driver's brief is the 0x250-class start in a real-pace walk
  (my walk wedged to frame 18943; the frame number itself is pace-dependent).

## What is left

1. **Why stream 2's op-1 never fires.** Next step: mid-scene, print `Script::stream_bindings()`
   for 0600007f (kind, obj, first/last key per stream — the read-only probe accessor exists) and
   log the drained `Request::AnimSound`/`PreCache` ids. If stream 2's kind ≠ 2 or obj is None,
   `anim_start` (nis.rs) never ran its `Cmd::Anim` — chase the stream dispatch for streams 1 vs 2
   (identical shape [D]; keys 94..=95 vs 96..=97). Compare with the game: `AnimProcessScriptCmds`
   case 1 fires on (old, new] with the 0x33 precache window [M 0x1c1b38].
2. **The disc-length read-back the brief asks for.** Done for 0x250 on the null mixer (full 77952).
   0x253 needs the same read once it starts. Note the port's `stream_data` takes the FIRST type-9
   marker; both headers' marker 0 is type 10 (pos 0) and marker 1 is type 9 — correct entries here.
3. The "may stop before it ends" symptom did not reproduce for 0x250 in two runs. If the main
   agent's real-pace run shows an early stop, the sound log (`NF_SOUND_LOG`) gives the exact
   start/stop/kill lines to cite.

## Live checks for the main agent

- In PCSX2 at the bath scene (07000002, ch 0x35 rise): a gdbwatch on the IOP's stream-start RPC
  path is not practical; instead **listen**: the game plays TWO lines in the scene (the 0x250
  speaker at ~t2, the 0x253 speaker at ~t286 — 3.5 s and 3.9 s per the LUT headers). If the game
  plays only ONE, the port's missing 0x253 matches the game and the divergence is elsewhere.
- `fieldprobe` screenshot at the scene's t≈300: the second character (skin 0x05000009) should be
  on screen mid-gesture — if the port's anim object is missing its pose is a draw-side symptom;
  the task is sound-only, so this is context, not a fix target.

## Usage

Token, credit and context-window figures: `not visible to me` (this session exposes no token,
cost or context meters).
