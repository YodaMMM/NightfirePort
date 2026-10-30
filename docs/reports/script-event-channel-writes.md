# script-event-channel-writes — the 0xD/0x11 channel-write events: the can't-clear rule landed, the disc tests, the 16-walkreal sweep

## What was asked

The port parses `Cmd::Event` (op 0x12 [M 0x1d8548's stream walk]) but the driver brief said it
acted only on kind 0xB, with `Script_EventHandler` 0x1d94c0's 0xD/0x11 channel-write events
ignored. Port both events exactly as the decompile does (which channel table, set vs clear,
timing within the frame), unit-test each event on real disc scripts, then run every walkreal
binary whose levels carry these events (sweep the discs) and report each total.

## What the decompile actually says [M]

`Script_EventHandler` 0x1d94c0 (decompile + disassembly 0x1d973c-0x1d97bc, raw words verified):

- **Case 0xD** (params [ch, value]): `ch == 0` → return (0x1d9740 beqz); **`value == 0` (u32) →
  return TOO** — 0x1d9758 `beqz $v1` fires BEFORE the `sb` store at 0x1d975c (raw words
  `8fa30004 10600034 a0440000`). Else `switch_channels[ch] = (byte)value` [M 0x1d975c] and
  `switch_channels_time[ch] = GameState._52_4_` [M the 0x1d97a4-0x1d97bc join — unconditional
  behind the value gate]. The table is the GLOBAL `switch_channels` byte array (base 0x26FC90)
  with the u32 `switch_channels_time` stamp array (base 0x270190).
- **Case 0x11** (params [test, dest_on, dest_off, value]): reads `switch_channels[test]`; dest =
  p1 when it reads nonzero, p2 when zero (0x1d977c-0x1d9784); `dest == 0` → return (0x1d9788
  beqz); **`value == 0` (u32) → return TOO** — 0x1d9798 `beqz $a0` before the `sb` at 0x1d979c
  (raw words `10800024 a0620000`). Else `switch_channels[dest] = (byte)value` + the stamp on
  dest.
- **The rule both cases encode: the script channel-write events can SET a channel, never CLEAR
  one.** A zero value returns before any store — no array write, no time stamp.

The decompiled C drops BOTH zero-value returns and shows the writes unguarded — the prior
`script-channel-events` task (8066bfa2, merged) ported the arms from the C and pinned the wrong
reading ("the write happens, value 0") in `event_0xd_writes_the_channel_with_the_games_time_
stamp_rule`. The disassembly is authoritative; this task corrects the port and that test.

## What landed

1. **The arms** (`crates/nf-game/src/nis.rs` event(), the 0xD/0x11 arms): gated on the u32 value
   != 0 exactly per the disassembly (ch/dest != 0 AND value != 0 before ANY write or
   `script_channels` registration; the (byte) truncation and the frame stamp unchanged). The
   disc's 199 census rows carry no value ≥ 0x100 [D census_channel_events.rs], so the u32-gate /
   byte-write edge (a p1 of 0x100 writes 0 WITH a stamp per [M 0x1d9758→0x1d97bc]) is
   disc-unreachable; the arms match the reachable rule exactly.
2. **The corrected lib test** with the address (the driver's fix-the-test rule):
   `event_0xd_writes_the_channel_with_the_games_time_stamp_rule` now pre-sets ch 0x1e = 9 and
   pins that the [0x1e, 0] event leaves it at 9, un-stamped and unregistered [M 0x1d9758 beqz],
   instead of the old "wrote the 0" pin. The 0x11 test gained the p3 == 0 case (pre-set 0x52 = 5
   survives a [7, 52, 53, 0] event untouched) [M 0x1d9798 beqz]. Both cite the raw words.
3. **The per-event unit tests on REAL disc scripts**
   (`crates/nf-view/tests/script_channel_writes_live.rs`, opt-in NF_ISO): the driver's cited rows
   decode from the ISO's own archives and each drives through the real `Script::update` path:
   - 07000012 / 0x06000530 / stream 8: four event-3 holds [M 0x1d94c0 case 3 — p0 = 0 → the
     delay −1 hold] then **0xD [3c, 3c] at t520** → ch 0x3c = 60, stamped at its frame;
   - 07000012 / 0x06000a25: stream 2 **0xD [89, 1] at t2** → ch 0x89 = 1; stream 3 **0xD [89, 0]
     at t4 → NO write at all** (the can't-clear pin on the real bytes);
   - 07000005 / 0x06000952 / stream 8: **0xD [62, 1] at t93** → ch 0x62 = 1, stamped;
   - 07000005 / 0x06000a23 / stream 1: the disc's ONE kind-0x11 row, **[98, 98, 1d, 1] at t300** —
     the test channel 0x98 reads zero on this path → dest = p2 = 0x1d → ch 0x1d = 1, stamped
     [M case 0x11].
   The isolated-stream runner applies the game's own event-3 un-hold each frame (SP_Activate
   0x1dc338:84 / SP_UnPause 0x1dca58:13 zero SCRIPTINFO+0x9c [M]) because 06000530's stream
   carries its own holds.
4. **The disc sweep** — `census_channel_events.rs` re-run: **199 rows across 15 of the 23 on-foot
   levels** (01, 02, 03, 04, 05, 06, 07, 08, 0b, 11, 12, 14, 16, 1b, 4a; 09/0c/0d/13/15/41/4b
   carry none), with **14 zero-value rows** (07 [82,0]; 0b [77,0]×2, [76,0]; 12 [89,0]; 14 [92,0],
   [b8,0]; 16 [50,0], [f3,0]×2, [b4,0], [8d,0], [94,0]; 1b [18,0], [1c,0], [12,0]×2) — every one
   of them a write the OLD port performed and the game never does.

## The walkreal sweep (every level with census rows) — totals

All run `--test-threads=1` on this branch's final tree:

| binary | total |
|---|---|
| walkreal_07000001_live | 5 passed; 0 failed (51.92 s) |
| walkreal_07000002_live | 1 passed; 0 failed (52.03 s) |
| walkreal_07000003_live | 4 passed; 0 failed (393.68 s) |
| walkreal_07000004_live | 7 passed; 0 failed (6.77 s) |
| walkreal_07000005_live | 10 passed; 0 failed (72.59 s) |
| walkreal_07000006_live | 3 passed; 0 failed (43.53 s) |
| walkreal_07000007_live | 2 passed; 0 failed (11.06 s) |
| walkreal_07000008_live | 3 passed; 0 failed (105.81 s) |
| walkreal_0700000b_live | 3 passed; 0 failed (42.85 s) |
| walkreal_07000011_live | 1 passed; 0 failed (9.74 s) |
| walkreal_07000011_route_live | 1 passed; 0 failed (2.40 s) |
| walkreal_07000012_live | 1 passed; 0 failed (5.42 s) |
| walkreal_07000014_live | 6 passed; 0 failed (57.14 s) |
| walkreal_07000016_live | 8 passed; 0 failed; 1 ignored (38.96 s) |
| walkreal_0700001b_live | 8 passed; 0 failed; 1 ignored (13.81 s) |
| walkreal_0700004a_live | 1 passed; 0 failed (6.74 s) |

**The one walk the new writes changed** — 07000016's `the_real_order_launch_chain_matches_the_
teleported_one`: its pin asserted the launch cinema's [180, 0] event CLEARS ch 0xb4 (door[4]'s
panel channel). The game never clears [M 0x1d9758 beqz before the sb]; the test was corrected
with the address and the raw words, and now pins ch 0xb4 staying on [D trigger #270]. No other
walk moved (the zero-value events' channels are otherwise unread on the walked routes, and the
set-value rows behaved identically before and after).

## How it was verified

- `cargo test --offline --release -p nf-game --lib nis::tests::event_ -- --test-threads=1`:
  **5 passed; 0 failed** (the two corrected + the 0xB/0x9/skip pins).
- `cargo test --offline --release -p nf-view --test script_channel_writes_live -- --test-threads=1`:
  **5 passed; 0 failed** (the real-disc per-event tests).
- `cargo test --offline --release -p nf-view --test script_channel_events_live -- --test-threads=1`:
  **3 passed; 0 failed** (the prior task's all-level write audit + the searchlight play/skip pins
  still hold under the corrected rule).
- `cargo test --offline --release -j2 -p nf-game -- --test-threads=1` (the whole touched crate):
  **all result lines ok, 0 failed** (44+ tests across 20 binaries).
- The 16 walkreal binaries above (58 passed, 0 failed, 2 pre-existing ignores).
- The full four-crate suite (`tools/test/suite.py`): this sandbox kills background processes
  (verified again this session — the nohup runner died between tool calls with an empty log), so
  the fast-runner total could not be produced; the touched-crate evidence above stands in.

## What's left

- The 14 zero-value rows' READERS (what each level's triggers/doors would have listened for had
  the game cleared) are [D]-listed in the census table; none is reachable on the walked routes
  (the sweep above), and the game's rule says the writes never happen — the reader hunt is
  informational only.
- The u32-gate/byte-write stamp edge (a value ≥ 0x100) is [M]-pinned by the disassembly but has
  no disc instance to test through; the arms implement it by construction of the gate.

## Live checks for the main agent

1. 07000016 fresh boot through the launch chain (the walked order): after the launch cinema,
   door[4]'s panel channel (ch 0xb4 = 180) must still read 1 (gdbwatch `switch_channels[180]`
   base 0x26FC90 + 180 = 0x26FD44 across the cinema's t747). The port now pins the same.
2. 07000012 fresh boot at the 06000a25 scene: `switch_channels[137]` (0x26FD39) goes 1 at the
   scene's t2 and NEVER returns to 0 (the [137, 0] event at t4 writes nothing) — gdbwatch for
   any store of 0 there.
3. 07000005's EndLevel script: `switch_channels[0x1d]` (0x26FCAD) must rise only when
   `switch_channels[0x98]` (0x26FD28) reads 0 [M case 0x11's dest selection] — a poke of 0x98
   to 1 before the scene should redirect the write to ch 0x98 (p1) instead.

## Usage

not visible to me — this session's environment exposes no token, cost or context-window figures
for the main session. The only usage figures visible are the per-call token counts my sub-agents
report in their result footers (e.g. the disassembly-research call: prompt 29,530 / completion
5,501 / total 35,031; the nf-game suite runner: prompt 8,445 / completion 588 / total 9,033 —
read from those results). No cost or credit figures anywhere.
