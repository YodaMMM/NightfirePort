# WORKLOG

## level-02-bath-line (2026-11-08, UTC)

Branch `s/level-02-bath-line` cut from `origin/master` (b059df948). Task: in the bath scene
(about frame 4882) the servant's spoken line may stop before it ends. A fix for the same
symptom in the opening scene landed (starting a voice now clears a stale end flag, 74ee9b749).
Run the app path with NF_SILENT=1 NF_NULL_MIXER=1 NF_SOUND_LOG=1 (the real mixer at real
time, no audio device) through to the bath scene, read the line's start/stop and played
samples against its disc length, and fix any remaining cause. Headless tests run the sim
ahead of the mixer and starve streams; do not use them as evidence for stream length.
Reproduce on the real level first through real input, not a teleport. Compare with the
decompile, fix, and test red on the old code. No draw changes.

Entry started — decompile reading follows.

Progress (22:50 UTC): the bath scene is SP #399 (script 0600007f, pack 07900002). Its spoken
lines ride the two Anim streams' op-1 sfx: 0x250 (6000502/skin 5000034, scene tick 2) and
0x253 (6000501/skin 5000009, tick 286) — both in the level bank (slot 11), both streamed:
0x250 → STREAMS.LUT[285] (44 544 ADPCM bytes = 77 952 samples = 3.54 s), 0x253 → LUT[284]
(48 896 bytes = 85 568 samples = 3.88 s). The NIS Sound ops 418/508 are water barks (418 not
in the bank). Repro (bath_line_repro): real walked route to the scene, the app sound path
(null mixer, real time). Run 1: 0x250 keyed on at the scene's tick 1 via the precache
takeover and played its full 77 952/77 952 — no early stop; the run's log window closed at
scene tick ~286 (0x253 unobserved). Extending the paced window.

Progress (23:05 UTC): run 2 paced the whole scene (740 ticks, frames 18944..19684). Line 0x250
again full length (key-on f18945, end flag f19153 at 208 updates, kill "sample-ended" — the
mixer saw 77952/77952). Line 0x253 NEVER reached the driver: no PreCache, no Start in the
whole log — while both anim scripts sit in sim.scripts with their op-1s (06000501 op1 [286,
595], 06000502 op1 [2, 592] [L port runtime]). The divergence is upstream of the driver:
stream 2's case-2 (Obj::Anim tick) never emitted the op-1 requests. Next step: log
Script::stream_bindings() for 0600007f mid-scene (kind/obj per stream) and the request drain.
