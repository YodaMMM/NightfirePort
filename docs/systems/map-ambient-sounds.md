# The map ambient sounds and the IOP voice budget (bathhouse-voices-02b)

Build: ACTION.ELF `e7ffdcb3…`, SFX.IRX in `work/decomp/SFX.IRX/`. Tags: [M] code (function@addr), [D] disc
bytes, [L] EE snapshot, [S] suggested. Written for playtest 1017-1139's "no guard voices under the
bathhouse" follow-up: lead 1 (the IOP voice budget/priority reject vs the guards' shout) and lead 2 (the
waterfall ambient, map-ambient id 92, whose start emitted no Play in 776's headless runs). The speech
machinery itself is docs/systems/speech.md + docs/systems/under-bathhouse-voices.md (776).

## Lead 2 — why the waterfall never played: the port read the wrong map's block 0x28

`GMapSounds`/`GNumMapSounds`/`GMapSoundHandle`/`GMapSoundActive` (0x2d3618…) are GLOBALS, not per-map:
`Sound_LoadMapSounds` 0x1ddff8 **resets** `GNumMapSounds` and the 0x31-byte active-flag run, then copies
its block's 32-byte records in. It is called from `parsemap_handle_block_id` 0x1d1020 case 0x28 (only on
the create pass, param ≠ 0) for **every** block 0x28 the archive parse walks — `parsemap_parsenextblock`
0x1d12e8 → `parsemap_handle_block_id`, driven by `parsemap_parsemap` 0x1d1cf0's whole-archive loop in
archive order. So after a level load, `GMapSounds` holds the **last** archive map's block 0x28 [M].

07000002 carries two: map 17's degenerate single record (id 0 at (0,0,0), radius 0) and map 51's four
[D `Level::parse` block 0x28, both lists printed by the parse probe]:

| # | id | pos | start radius (outer) | inner |
|---|----|-----|------|-------|
| 0 | 0x40000024 → 36 | (−55.69, 6.32, −16.08) | 84.19 | 75.77 |
| 1 | 0x40000029 → 41 | (−55.79, 8.91, 2.46) | 96.67 | 87.00 |
| 2 | 0x5c → 92 (the waterfall) | (−54.96, 0.00, 1.01) | 91.00 | 81.90 |
| 3 | 0x40000087 → 135 | (−50.51, 8.91, 4.48) | 96.67 | 87.00 |

The port's `MapSounds::new(level.maps[main].sounds.clone())` took the **main map's** list — empty on
07000002 — so no map ambient ever started there; the waterfall's "start emitted no Play" in 776 was a
start that never happened. **Fix**: `Level::parse` records the last-wins list as `Level.map_sounds`
(the game's global), `Drones` builds `MapSounds` from it.

The rest of the object matches the game:
- Start gate: `HandleMapSoundAllocation` 0x1dd118 (run by `Sound_UpdateListeners` 0x1dd278 before
  `Sound_UpdateSounds`) starts an inactive entry when the listener's squared distance is inside the
  record's radius², calling `Sound_Play3D(100.0f, pos, …)` — the entry is active until the squared
  distance exceeds **1.2 · radius²** (`Sound_Stop` + flag clear) [M 0x1dd118]. The port's
  `MapSounds::update` is the same gates [M crates/nf-game/src/sound.rs].
- Radii: the game passes the custom pair only through the node's custom flag (`Sound_Play3D` 0x1dd548
  sets node+0x30 = 1 when a caller radius > 0; `Sound_ReqestPlaySfx` 0x1dcc68 then sends node+0x40/+0x44
  as `SFXStart3D`'s inner/outer, else −1, −1 → the bank's own radii). The port sends the pair
  `(record inner, record radius)` as (inner, outer) [M sound.md §1; the 0.9·radius pattern above holds
  for all four records [D]].
- Bank effects on 07000002 (hash 2 → SBINFO slot 11) [D SB_11.SFX]: 36/41/135 are tracking 1, priority
  100, looped single samples (111/107/198, volumes 80/80/60); **92 is tracking 0 (2-D), priority 100,
  `multi_sample` 1 + `polyphonic` 1, count 1** — the polyphonic arm of `SFXSetup` queues a cue timer and
  starts NO sample at setup (`PolyPrintf` branch, 0x4220:168-176); the voice keys on when `SFXUpdate`
  0x4ee4's polyphonic arm fires the cue (the port's poly arm matches: `csw[k]==0 → StartSample`).
  Sample 207 (74 944 B, header loop bit 1) sits fully inside the SBF [D].

## Lead 1 — the game's voice-allocation rule, and the shout's margin

`SFXSetup` 0x4220 (the IOP side every start goes through) [M], with `CountAllVoicesActive` 0x40ec (other
items' ES voice handles), `SFXGetImportance` 0x41e0 (= SFXParameters+0x14 = priority) and
`ES_FindFreeVoiceHandle` 0x88 (100 ES handles, first free):

1. **Budget**: `budget = 15 − startedCounter/400` (the counter is clamped to 2000 at setup and decays
   ×9/10 per `SFXUpdate` 0x4ee4; each setup adds 100 — winnable back within seconds).
2. **3-D reject** (only `tracking == 2 && priority < 100`): reject when `dist > outer`, else when
   `proximity · priority < activeVoices · 2000 / budget` where
   `proximity = clamp(100 − (dist−inner)·100/(outer−inner), 0, 100)`.
3. **Budget steal** (`activeVoices > budget`): mark `(activeVoices − budget)` lowest-priority items
   (skipping deleted, streamed and tracking-3 items; the calling item competes too), then: a
   `max_reject ≠ 0` sound rejects itself instead of stealing; an item the pass marked rejects silently.
4. **Per-id limit** (`max_voices ≠ 0`): count other live items with the same id (SFXItem+0x8
   HashCode); over the limit, a `max_reject ≠ 0` sound rejects itself, else the **oldest** sibling
   (SFXItem+0xC Counter, +1 per `SFXUpdate` [M 0x4ee4]) is marked.

The port's `Driver::setup`/`update` implement exactly these rules (compared line by line: budget clamp
and decay, the reject formula's integer expression `(n·budget·2000)/(budget·budget)`, the steal scan
including self, the `max_reject`/self-marked returns, the per-id oldest steal, `SFXGetImportance` =
priority, the polyphonic/sequential/shuffled sample choice, the ducker gate) — **no deviation found**
on any rule the decompile states. The shout's margin on 07000002 [D SB_11.SFX: ids 111..115 are
tracking 2, inner 2, outer 50, max_voices 5, priority 99, ducker 10; sub = one stream (files −135..−139
→ STREAMS.LUT 134..138), volume 100]:

- with the four ambients live (4-7 ES voices) and budget 15: threshold = 4·2000/15 ≈ 533 ≪ 100·99 = 9900
- even at 20 active voices (budget 15): 2666 ≪ 9900; a guard metres away has proximity 100.

So the game does not reject the shout here, and neither does the port; the remaining difference between
776's headless chain and an audible playtest is only the output device (streams report their ends; the
mixer mixes). Tests pin the outcome on the real path (`bathhouse_shout_mixer_live`): the shout's stream
Play (pitch 0x759 = 1881 [M `StartSample` 0x3524]) reaches the captured batch at non-zero volume while
the four ambients keep their voices.

## What reaches the mixer, per id (for the main agent's live comparison)

On the real path (`nf_view::load` + `Sim::step_bond` + `nf_view::sound_tick`, the batch commands the
output would receive):

- **Ambients 36/41/135**: one Play each at pitch 941 (bank samples 111/107/198), front volumes ≈
  0.18..0.32 at the intro's listener spot, plus the rear pair voice (left = rear level, right
  phase-inverted — 0.0/−0.0 at that spot) [M driver `levels`/`PS2_SFXCalculate3D`].
- **Waterfall 92**: no Play at start (the polyphonic arm only arms the cue timer); its Play comes when
  `SFXUpdate`'s cue fires, same pitch 941, non-zero volume.
- **Shout 111..115**: one Play at pitch 1881 (STREAM_PITCH), volume = the stream's ES_CombineVolumes
  × 3-D attenuation at the guard's position; the rear pair follows.
- All four ambients keep their voices through the shout (the steal never reaches them: 4-8 voices vs
  budget ≥ 10, and their priority 100 tops the steal scan).

## Live checks for the main agent

1. PCSX2 07000002, fresh boot, tunnel route to plate #389 (−47.0, 6.0, −23.3): with the ambients fixed,
   EXPECT the waterfall (and the three pool-area ambients) audible near the underground pool, and the
   trio's shouts audible when they engage.
2. If any shout is still inaudible live: gdbwatch `SFXSetup` 0x4220's reject branch (the return-0 path
   at 0x4220+~0x150) on the shout's item — EXPECT not taken (per the margin math above); and
   `ES_CombineVolumes`' result for the shout's psiSFX — EXPECT > 0.

## Tests

- `crates/nf-view/tests/bathhouse_map_ambients_live.rs` — the parse pin (`map_sounds_are_the_last_block_0x28_not_the_main_maps`,
  sister: the degenerate earlier block did not win) and the real-path pin (all four ambients key on,
  non-zero 941-pitch front volume each, and keep their voices; `looping_voice_ids` refines the headless
  end-all stand-in to keep looped samples, which never report their end on hardware).
- `crates/nf-view/tests/bathhouse_shout_mixer_live.rs` — the bunker trio's shout reaches the mixer
  (stream Play, non-zero volume) with the ambients live; sister: the ch-0x15 floor five through the
  same chain.
