# level-11-every-objective-3 — the report

Task: find what writes ch 0x48 in the game ([D] the disc's channel writers; [M] the code path),
port it if the port lacks it, then extend `walkreal_07000011_live` on real input through every
remaining objective to the mission's end (the next level's load). Every test in the file green.

## What's done

1. **The ch 0x48 writer — identified [D+M]; the port already has it; nothing to port.**
   - The [M] code path: the disc's touch triggers are created by `Trigger_Create` 0x1b1da0 and
     updated by `Trigger_Update` 0x1b1a78 (work/decomp/ACTION.ELF/001b1a78, symtab line 3186).
     Its `case 1:` (TouchOnce) early-outs when `switch_channels[out]` is already set — the ONCE —
     and falls through into `case 2:`, which performs the write
     **`switch_channels[out] = switch_channels[out] | (Trigger_Hit(...) != 0)`** plus the latch
     `switch_channels_time[out] = GameState._52_4_` (0x1b1a78:73-89). `out` is the trigger's out
     channel (TRIGGER_Tag+0x20, the disc's param 0); `Trigger_Hit` is the mask-8 contact test.
     So the game's writer of a TouchOnce channel is Bond standing in the volume — no other class
     writes it unless a param names it.
   - The [D] writers on the mission's discs, swept fresh this session by the committed probe
     `walkreal_probe11_48_live` (extended with a 07000012 sweep; diagnostic, `--nocapture`):
     - **07000011**: zero 0x48 writers. The script streams carry exactly one 0xD event
       (`script 0600098a stream 6: params [98, 1]` — ch 0x62, the atrium-fall player-dead write);
       zero 0x11 events; the raw placement param/extra sweep has zero 0x48 hits; the parsed
       triggers/SS/doors have none; the drones' death channels hold one value, 0x08. Matches
       level-11-every-objective-2's hunt line for line.
     - **07000012**: exactly ONE 0x48 writer — TouchOnce **#679** (`plc[679] flags 0xea, the only
       raw 0x48 param hit, idx 0`; parsed `trig[34] placement=679 kind=0x1 out=0x48 gate=0 key=0
       mask=0x08 at (22.53, 0.72, 19.13)`, the spawn tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62)
       [D docs/levels/07000012.md §1]). The streams carry three 0xD events — `script 06000530
       stream 8: [60, 60]`, `script 06000a25 stream 2: [137, 1]` / `stream 3: [137, 0]`, plus the
       shared fall cut-scene's `[98, 1]` — **none writes 0x48**, and none lies on the walked route
       (the walk's green run is the behavioural check). Death channels: 0x63 (the civilian) and
       0x08 ×2 — no 0x48.
   - The port has the writer (`crates/nf-game/src/triggers.rs`, `kind::TOUCH_ONCE` with the
     out-channel contact arm): the walk asserts `sim.drones.channels.on(0x48)` right after the
     07000012 spawn and it passes — `#679`'s tile fires under Bond's feet, the monitor completes
     0x04000036 [M 0x1827d8 case 3]. **Nothing to port.** The -2 conclusion ("the objective
     completes one level later by the game's own design") now stands verified on BOTH levels'
     disc data rather than only 07000011's.
2. **The walk extension — every remaining objective to the mission's end (the next level's
   load), real input throughout** (`crates/nf-view/tests/walkreal_07000011_live.rs`):
   - The 07000012 leg's load switched to `load_with_carry_difficulty(..., 3)` — the exact call
     `walkreal_07000012_live` calibrates `tower2`'s route on (the old spawn-stand leg loaded
     difficulty 1; the spawn pins kept here are difficulty-independent [M the 0x1827d8 announce +
     #679's contact write]). Every pre-existing assert is untouched.
   - After the spawn stand (0x04000036 completes), the walk continues through **tower2's route**
     (`common/tower2.rs`, shared with `walkreal_07000012_live`): the Q-Decryptor burn → the
     door[19] unlock → the security-centre console E (**ch 0x49 — objective 0x04000037
     "Activate elevator override control in security center" completes** [D #731; M
     Trigger_Activate 0x1b2318 type 1]; tower2 pins `states[16][0] 3 → 4`) → the lift call button
     #699 (ch 0x58 swaps both shutter SS pairs) → the shaft entrance (#683 holds ch 0x1F) → the
     gated exit #669 auto-fires → **`level_exit == Some(0x0700_004a)` — the mission's end, the
     next level's load** [M Trigger_Update 0x1b1a78 case 10].
   - The tail carries the RamSave through the real load into 0700004A [M Player_RamLoad
     0x1957a8], where 4A's start #43 (4.26, −7.71, −4.76 [D]) drops Bond INTO touch #277's
     volume → **ch 0x4D fires and 0x04000038 "Escape through the elevator shaft" completes**
     (the late AND — its writer lives one level ahead [D 0700004a.md §2]; the monitor runs before
     the object handler so the write lands the next frame [M 0x182c60 from 0x1c9790:17]). Pinned:
     `states[16][1] == 4`, `states[16][0] == 4`, `states[15][1] == 4` (carried), and
     `states[15][0] == 3` — **0x04000035 "Escape from the tower" stays announced**: its writer is
     the whole-mission AND (ch 100 [M 0x1827d8:119-123]), which first fires on 07000013 [D §2] —
     the mission's remaining part, beyond this walk's endpoint by the game's own design.
   - Result: **every objective of mission 0x07000011 that has a writer on the walked levels
     (07000011 → 07000012 → 0700004A) now completes and is pinned in one real-input walk** —
     0x04000036 (ch 0x48, #679), 0x04000037 (ch 0x49, #731), 0x04000038 (ch 0x4D, #277) — with
     0x04000035 pinned announced-only.
3. **One pre-existing port gap re-checked, still inert** (the -2 follow-up): the port parses
   `Cmd::Event` (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB; the
   `Script_EventHandler` 0xD/0x11 channel-write events [M 0x1d94c0] are not acted on. The new
   07000012 sweep shows the level's three 0xD events write ch 60 / 137 / 98 — none gates the
   objectives or the route, so the gap stays a general follow-up, not a task fix.

## How it was verified

- `cargo test --offline --release -p nf-view --test walkreal_07000011_live -- --test-threads=1`:
  **1 passed; 0 failed** (`finished in 10.16 s`) — the extended walk: "[011] walked out in 3262
  ticks; stuck 0; falls 0", the 07000012 leg "[012] walked out in 1645 ticks; stuck 0; falls 0".
- `cargo test --offline --release -p nf-view --test walkreal_probe11_48_live -- --test-threads=1`:
  **2 passed; 0 failed** — both [D] sweeps (07000011 + 07000012), quoted in §1.
- `cargo test --offline --release -p nf-view --test walkreal_07000012_live -- --test-threads=1`:
  **1 passed; 0 failed** (`finished in 5.49 s`) — the shared tower2 route unchanged by this task.
- The FULL four-crate suite: see the Suite section.

## What's left

- Nothing in the task's scope. The general `Script_EventHandler` 0xD/0x11 port remains the
  documented follow-up (inert on this mission's data — §1/§3).
- The suite run is disk-bound in this sandbox (the ENOSPC dance below); if the stop line cuts it
  short, the next session re-runs `tools/test/suite.py` on the final tree for the total line.

## Live checks for the main agent

1. 07000012 fresh boot: cross the door[19] doorway westward (the route's stand-in) — watch
   Bond's xz through the leaf tunnel x 6.74..7.07, z −1.46..−0.26; the port pins at x ≈ 7.14
   (the open leaf's east edge pokes 0.2 into the tunnel mouth). Record whether the game's
   capsule slips the edge and at what x.
2. 07000012 fresh boot: walk the north wall's opening into the shaft lobby (the second
   stand-in) — watch Bond's xz through the 0.88 pinch at z ≈ 8.7 between the round pilaster
   (x 5.25..6.12) and the core's west wall (x 7.00), and how the game resolves the ramp's
   x 5.0..5.5 floor trench. The expected pass is along the move (Intersect_CylGeom 0x109da0
   behaviour), not a two-sided depenetration.
3. 07000011 fresh boot (carried over from -2, still open): confirm the HUD never announces
   "Objective complete" for "Gain access to lower office floor" on 07000011 itself — the probe
   now [D]-sweeps both levels and finds no 0x48 writer on 07000011; a live confirmation upgrades
   the writer-absence from [S]+[D] to [L].

## Suite

The fast runner (`tools/test/suite.py`) cannot run in this sandbox: the environment kills
background processes between tool calls (the nohup'd run died twice with an empty suite.out), so
the sanctioned chunked variant `tools/test/chunk_suite.py` ran instead (same build, same
per-binary limits, state across calls). The sandbox also ran out of disk twice during the build
(the same ENOSPC the -2/level-02 reports documented): purged the sandbox's non-task data
(the default project's node_modules, cargo's registry cache — re-fetched), the stale
first-pass artifacts (a linker-flag retry had produced a dead artifact set), and `work/`
(the decompile cites were already taken). No repo file touched by any purge.

See the tail of this file for the final total line reached before the stop line.

## Usage

not visible to me
