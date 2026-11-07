# unread-params-switches — report

Branch `s/unread-params-switches` (from origin/master 10956badb). Task: for every switch /
trigger / sensor / touch / channel-logic class created by `parsemap_create_dynamic_objects`
0x1d04f0, list what each `*_Create` stores, compare with the port, census every ignored param on
all 34 levels (`Level::parse` over `nf_view::dev::LEVELS`), port the real gaps, test on the real
level, red on the old code.

## The class sweep (all `*_Create` stores checked against the port)

| class (0x1d04f0 case) | `*_Create` | params stored [M] | port | verdict |
|---|---|---|---|---|
| 0x29 Switch | `Switch_Create` 0x1adcc0 | p0→ext+0x12, p2→+0x10, p3→+0x16, p4→+0x19, p5→+0x14, p1→the embedded SP (`SP_Create` :44-46), **p6→`SSysItems[p6·8+4] = the switch's channel` (:39-41)** | useobj.rs reads p0..p5 + the SP wiring; **p6 was unread** | **p6 ported this task** (below); p0-p5 closed |
| 0x35 PCQWorm | `PCQWorm_Create` 0x1b07a8 | ext[0..2]+ext+5 = p0..p3, nothing else | useobj.rs reads p0..p3 | closed — no 4th+ param exists [M the create] |
| 0x28 Sensor | `Sensor_Create` 0x1af010 | p0..p7 (lens, keys, range, out ch, disable ch, p5 camera-list byte, monitor group, mount) | sensor.rs reads all; p5 documented as the CameraList join byte | closed; **p5 is zero on every level** [D census] — doubly closed |
| 0xdc raw Trigger | `Trigger_Create` 0x1b1720 | out=p1, inputs=p2..p8, mask=p9 (0→2), lock=p10, hide=p11 | triggers.rs `apply_raw_dc_map` | closed (census: 0700000c's three raw rows exercise p0/p1 only) |
| 0xea/0xeb Touch wrappers | 0x1b2b88 / 0x1b2aa8 | slot9=p1 (mask), slot10/11 = p7/p8 (once) and p6/p7 (touch) — re-verified against the decomp this session | triggers.rs `from_params` matches arm for arm | closed |
| 0xec/0xed/0xee/0xef mux | 0x1b2c68 family | out=p0, inputs=p1..p6+p8, mask=p9, lock=p10, hide=p11 | triggers.rs mux arm | closed; **p7..p11 zero on every level** [D census] — the mask always defaults to 2 |
| 0xe8 LoadLevel | 0x1b2950 | next_level=p0, key=p1, gate=p2, fallback=p3, success=p4, mask forced 0xE | triggers.rs | closed |
| 0xf4 Movie | 0x1b2a08 | mask = p1 (slot 9 = old p1), out = p0 | triggers.rs | closed (07000041's p1=4 reads as mask) |
| 0xf9 SoundTrigger | `SoundTrigger_Create` 0x1b2628 | p0 on-gate, p1 one-shot, p2 sound id (0/0xffff → no object), p3 2D flag, p4 OFF channel, p5 global-voice flag, **p6 the difficulty gate** (`p6 < difficulty → no object`; 5/6/7 → only difficulty 1/2/3), p7 keep-on latch | sound.rs reads p0..p7 incl. the p6 gate and the GlobalVoiceHandle slot | closed (the census's p3..p7 non-zero rows are all consumed) |
| 0xfb MusicTrigger | `MusicTrigger_Create` 0x1b30e0 | p0 gate ch, p1 kill ch, p2 `Music_Event` id (u32), p3 one-shot byte | worldobj.rs reads p0..p3 + the create gates | closed (07000003's p3=1 is the one-shot byte, read) |
| 0x41 Inverter | `InverterTrigger_Create` 0x1b3040 | p0 → +0xf4, p1 → +0xf6 | worldobj.rs | closed |
| 0x2e Lock | `Lock_Create` 0x1ae1f8 | p0 kind, p1 keycode, p2 (ext+12, u32), p3/p4 sound ids (0→0xffff), p5 out ch, p6 "unlocked at Operative", the keyframes from the placement's +2/+4 words | lock.rs reads p0,p1,p3,p4,p5,p6; **p2 unread** | closed; **p2 is zero on every level** [D census] |

## The gap found and ported: Switch param 6 — the SSys arm

`Switch_Create` 0x1adcc0:39-41: a switch with a non-zero param 6 writes its own channel (param 0)
into `SSysItems[p6·8+4]` — the security-subsystem slot's channel byte — at level load. The slot's
reader is `SSys_Monitor` 0x1ad998 (ticked from `Game_Run` 0x1c9790, only while `ScriptCam == 0`):
while the slot's channel is on it arms a 240.0 s countdown [L the ee_snapshot thresholds
MIN3/MIN2/MIN1/MIN01 = 180/120/60/6 @0x30cd84], announces each threshold (the SSys_Msg table
sounds), and at the expiry writes the channel back to **0** and disarms [M 0x1ad998 case 2]. Item
2 (laser 2) arms through the shared 8 s `SSysDelayTimer` and, when item 1 is also armed, forces
item 1's channel on and disarms item 1's monitor slot [M 0x1ad998:25-38].

**The port's `crate::ssys` machine existed but was DEAD CODE** — `git grep` on the base commit
shows no caller of `Ssys::new`/`monitor` outside ssys.rs's own tests, and nothing ever armed a
slot. The census: switch p6 is non-zero on exactly three on-foot levels — 07000009 (slot 1),
0700000a (slot 2), 07000046 (slot 1) — zero on all 31 others [D census].

Landed (crates/nf-game/src/drone.rs, ssys.rs, useobj.rs):
- `SwitchObj.ssys_slot` (param 6) stored by `UseObjects::create`; `Drones::attach_level` arms
  `self.ssys.items[slot].channel` from it after the switches are built (the game's store is
  unconditional on the channel value; slots ≥ 5 are the game's out-of-bounds writes — none on the
  disc — and the port no-ops them).
- `SSys_Monitor` ticks once per frame in `Drones::tick_inner` after the sound-trigger pass, gated
  on `scripts.camera().is_none()` (the `ScriptCam == 0` gate) — the same once-per-frame Game_Run
  pass the sound triggers ride. The monitor's channel writes are the game's RAW
  `switch_channels[..] = 0/1` stores (no `switch_channels_time` stamp [M 0x1ad998:31, 76]) — the
  port writes the value vector directly; `monitor` now returns the writes collected (the laser-2
  arm's mid-walk store has no same-tick reader, so collecting is behaviour-identical and kills
  the aliasing-closure problem), and the msg events play their table sounds through the audio tree.

The pin `crates/nf-view/tests/ssys_switch_arm_live.rs::the_armed_ssys_slot_counts_down_and_
switches_the_channel_off_like_the_game` (07000046, real load): after the switch's channel goes up
the arm msg sound (Laser1 id 6 = 0x2a2 [L ee_start_05 table 0x2bef60]) plays on the arm tick, the
channel is STILL ON at t=2000 (the countdown is not an instant reset), and the expiry switches it
OFF in the 14400-tick window (the 240 s arm ÷ REC_FRAME_RATE [M 0x1ad998:43]). **Red on the old
code**: the base commit has no caller of the machine (nothing ticks it), so the channel can never
return to 0 — the test's `off_at` would be `None`.

## The census table

Per level (34 rows, `nf_view::dev::LEVELS`), class × non-zero param counts — abbreviated
`cls×count [params param^count(example)]`; full output in the session log:

- **switch (0x29)**: 07000009 ×12 [p0^12 p1^12(SP id) p4^11 p6^1], 0700000a ×9 [p0 p1 p4^8 p5^1 p6^1],
  0700000b ×1 [p0 p1 p4], 07000014 ×2 [p0 p1 p4^2], 07000016 ×1 [p0 p1 p5], 07000046 ×8 [p0 p1
  p2^1(300) p6^1]. **p3 (rest) zero on every level** — `init_channels`' rest-arm is a no-op on
  all 34 [the port comment's claim checked on all levels, not one].
- **pc (0x35)**: 0a ×3 [p0..p3], 0b ×1, 11 ×1, 12 ×1, 46 ×1 — p0..p3 only, all read.
- **sensor (0x28)**: 02 ×2, 0a ×2, 0b ×2, 12 ×5, 14 ×14, 46 ×3 — p0..p4+p6+p7 read; **p5 zero everywhere**.
- **triggers (0xea/0xeb/0xec/0xed/0xee/0xef/0xe8/0xf4/0xdc)**: every non-zero param maps to a
  port-read slot; the mux families never carry p7..p11 [D census]; the raw 0xdc rows are only
  0700000c's three [p0^3(2) p1^3(27)].
- **sound (0xf9)**: present on 20 levels; p3..p7 non-zero on many (p4^4(101) on 07000014) — all
  consumed by sound.rs (the p6 difficulty gate included).
- **music (0xfb)**: 12 levels; p3 non-zero on 07000003 (the one-shot) — read.
- **invert (0x41)**: 06/12/14 — p0/p1 read.
- **lock (0x2e)**: 08/0a/0b/0c/11/12/14/15/46 — p0/p1/p3/p4/p5/p6 read; **p2 zero everywhere**.
- Driving/bonus maps 07000023-29/43/48/49/4b: none of these classes at all.

## How it was verified

- `ssys_switch_arm_live`: **1 passed** (2.4 s) — the pin above; red on the base (no machine caller).
- `nf-game --lib`: **688 passed, 0 failed, 5 ignored** (incl. the reworked ssys monitor tests and
  the useobj switch tests).
- `walkreal_07000009_live`: **6 passed, 0 failed** (7.2 s) — an SSys-armed level's walk with the
  machine live.
- `walkreal_0700000a_live`: **6 passed, 0 failed, 1 ignored** (26.2 s) — the other armed level.
- `npc_doors_live`: **8 passed, 0 failed** (14.0 s); `fix_guard_misc_live`: **13 passed, 0 failed,
  2 ignored** (48.9 s).
- Full suite (`tools/test/suite.py` + the resumable chunk runner): the sandbox kills background
  jobs between tool calls and the 200-binary release build alone outlasts a 10-minute chunk, so
  only **42 binaries / 256 tests** completed — 3 FAILED, all three reproduced byte-identically on
  a clean `origin/master` worktree with the shared target dir (`missing_scenery_live` ×2,
  `glass_guards_14_live::a_guard_behind_the_comp_room_window_is_visible_through_it`) — pre-existing
  on master, not from this branch (the diff touches no draw code; the three tests assert scene
  draws). Not run to completion: the remaining walkreal binaries (the diff's only sim-visible
  writes are the armed slots on 09/0a/046, and 09/0a's own walks pass; 07000046 carries no
  walkreal).

## What's left

1. The full-suite verdict on a sandbox that can finish the release build (the 3 known master-red
   draw tests aside).
2. The laser-2 special arm's channel-forcing write (item 2 expiring while item 1 is armed) has no
   disc level with BOTH a slot-1 and a slot-2 switch [D census], so it is pinned only by
   ssys.rs's unit test — a live check would confirm the 8 s delay on 0700000b's own wiring (the
   level's lasers arm their channels through scripts there, not through switches).

## Live checks for the main agent

1. **07000046's slot-1 expiry.** Fresh boot, flip the switch whose param 6 = 1 (the census row's
   p0 channel), wait 4 minutes of play: the switch's channel must return to 0 on its own
   (`switch_channels[ch]`), with the Laser1 msg sounds at arm/180/120/60/6. The port now does
   this; the pre-port tree left the channel latched.
2. **0700000a's slot-2 delay.** The same on the slot-2 switch: the arm waits behind the 8 s
   `SSysDelayTimer` first [M 0x1ad998:25-38] — the countdown starts ~8 s after the channel rises.

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy).
