# The SimpleScript sweep: one contact plays the whole travel (the 0700000c shot winches)

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] decomp / disassembly read here, [D] disc bytes, [L] EE snapshots,
[S] suggested. Port: `crates/nf-game/src/ss.rs` (`SsList::create` / `SsList::tick` / `Ss::sweep`). The
struct is the SIMPLESCRIPT the 0xE2 placements carry; `SS_Create` 0x1a9c58 builds it, `SS_Update` 0x1a9948
ticks it, `SS_Operate` 0x1a9df8 is its trigger test. A render check of the
0c build saw a shot winch stall at one step (t = 0.04, state 0x40, 120 ticks) —
this doc is the semantics that answer it, and the port's current shape.

## The field map [M SS_Create 0x1a9c58]

The ext block is a float-viewed struct (`pfVar2[k]` = +4k). `param k` = the placement's param word k;
`level_tag+2` = `extra[7] & 0xffff` (the key count), `level_tag+4` = `extra[7] >> 16` (the rate).

| ext | SS_Create source | meaning |
|---|---|---|
| +0x00 | `*puVar2 = 0` | the playhead `t` (key units), starts 0 |
| +0x04 | `(count − 1) / rate` (rate 0 → 0x4b = 75) | the per-tick step [M :51] |
| +0x08 | param 5, snapped `step * (int)(p5/step)` | the rest point (see the census — never inside a shipped sweep) |
| +0x0C | `*(level_tag+8)` | the 0x19 key-list pointer |
| +0x10 | param 3 | the channel word (an OUTPUT) |
| +0x14 | param 4, 0 → 0xffff | the operate sound id (`Sound_Play3D` at the 0x40 arm) |
| +0x1C | `count − 1` | the END KEY (the wrap bound) |
| +0x1E | param 2 | the operate mask (bit 1 channel? no — bit 1 = the bullet contact class; see `SS_Operate`) |
| +0x22 | param 0 | the state (0x40 armed / 1 playing / 8 looping / other hold) |
| +0x24 | param 1 | the wrap flag: 0 → the wrap re-arms 0x40, else disarms 0 [M 0x1a9948:119-128] |
| +0x26 | param 6 | the spline flag (0 → `KeyFrame_Interp`, else `Spline_Interp`) |
| +0x28 | 0 | the posed-this-tick flag (`obj+0xfa | 0x20` when set) |
| +0x2A | param 7 | the end-of-tick watcher write (`*(obj+0x20)+0x90 = obj+0x84`; obj+0x20 is 0 on the disc's SSs [S — no reader seen]) |

## The state machine [M SS_Update 0x1a9948]

`SS_Operate` 0x1a9df8 ORs the obj+0xD0 contact chain's class bytes (3 → 0x8, 2 → 0x10, 5 → 0x2
[M :12-45]) and ANDs the mask; mask bit 0 clear routes the channel half instead (the channel word ≠ 0
and `switch_channels[ch]` ≠ 0 [M :47-51]).

* **0x40 (armed)**: `operate == 0` → the tick does NOTHING (no pose, no advance [M :85]). `operate ≠ 0`
  → play the sound, **`ext+0x22 = 1`** [M :86-89], then the shared body runs THIS tick: pose at `t`,
  `t += step` (× `FRAME_RATE_MUL` = 1.0 at 60 Hz [M GS_SetRefreshRate 0x17d2b8:15; the 2-key case adds
  the step alone — identical at 1.0 [M :103-108]).
* **1 (playing)**: the shared body every tick — **no operate needed** [M :92-129]. The sweep runs to the
  wrap on its own. This is the answer: **one contact starts the full sweep** —
  state change, play to the end key, then the wrap.
* **8 (looping)**: pose, advance, wrap `t` to 0 past the end key, stay 8 [M :48-71].
* **other**: nothing.

The shared body's two exits [M :109-128]:
* the landing: `rest < t && |t − rest| < 0.0002` → sound stop + state 0x40 (frozen at t — the sweep
  halts there, re-armed);
* the wrap: `(int)end_key < (int)t` → `t = 0`, state = `wrap_flag == 0 ? 0x40 : 0`, sound stop. States
  0/0x40 never re-pose, so **the object holds its last posed matrix** — the visible end pose stays.

A re-arm (wrap_flag 0) is operable again: the next contact snaps the pose to key 0 (t = 0) and replays
the whole sweep [M the 0x40 arm reads t as it stands].

## The disc census [D, all 84 animated SSs (kind 0xE2, key count ≠ 0) on the 20 story archives]

* masks: 0x0/0x1/0x2/0x4 — **no SS carries 0x8 or 0x10** (the player/drone contact arms exist in the
  decompile but no shipped placement asks for them).
* wrap flags 0/1 both ship; nonzero channels: 1, 2, 8, 14, 18, 19, 40, 43, 49, 57, 84, 136, 137; sounds:
  0/0xffff (none), 283, 470, 584, 586.
* **rest (param 5) raw values: 0/15/30/45/60 — after `SS_Create`'s snap NONE lands inside its sweep**
  (the raw word is ticks-scale, the playhead is key-scale: e.g. 07000001 #16: step 7/50 = 0.14, rest
  45 → snapped 44.94, last key 7). The landing window is dead on every shipped SS — the wrap ends every
  sweep. (The `Ss.rest` field comment's old "every SS carries 0" was wrong about the raw values; the
  conclusion stands.)

## The 0700000c winches (#351/#364/#377) [D]

`obj_stor1_pully02` / `obj_stor3_pully01` / `obj_stor5_pulley`, all on map 45: state 0x40, wrap 0
(re-arm), mask 0x2 (bullet), **channel 0** (no channel half), sound 0xffff (none), rest 0, **spline 1**,
rate 75. Key count 4 → last key 3, step 3/75 = 0.04 per tick — 75 ticks from key 0 to the wrap. A shot
= one class-5 contact tick [M `Collide_Update` 0x1ea398:10-27 rebuilds obj+0xD0 per pass] → the latch →
the whole 75-tick spline sweep → the wrap re-arms 0x40 with the END POSE HELD (t = 0, the matrix stays
at the last key). A second shot replays from key 0. No channel ever moves them (the ch-0 poke test,
`fix_0c_vent2_live::the_pulley_winches_hold_key0_a_channel_0_poke_moves_nothing`).

## The port

`SsList::tick` mirrors the machine: the `0x40 if operate` arm sets `state = 1` then runs the shared
`sweep` (pose, advance, landing, wrap) [M]; state 1 sweeps contact-free; 8 loops; the channel latch
write [M 0x1a9948:18-23]; ch 0 is a real channel (fix-0c-vent2). The winch sweep's visible contract —
the end pose held after the wrap, the disc params above — is pinned by
`crates/nf-view/tests/fix_ss_operate_0c_live.rs` through the real fire path.

## Live checks (open)

- PCSX2, 0700000c, the storage balcony (winch #351 at (5.92, 14.50, −115.90), object
  `obj_stor1_pully02`): shoot the pulley once — the pulley should sweep over ~75 ticks (1.25 s) along
  its spline and HOLD the end pose. Render spots: `--bond 5.8,14.0,-109.5,<yaw toward the winch>,0`
  before the shot (key 0), at ~40 ticks (mid-sweep), and after 80 ticks (the end pose held).
- A second shot after the first sweep: the pulley snaps back to key 0 and replays.
- The sister: no shot — the pulley holds key 0 forever (state 0x40, no channel).
