# level-11-every-objective — 07000011's mission rows asserted on the real-input walk

Branch `s/level-11-every-objective` (from origin/master 284c1676, **merged with origin/master
87256135** — the batch-1028m every-objective batch — for the final tree; the merge was clean).
Sandbox 21:28–23:28 UTC, work stopped for the finish at 22:52.

## The coverage survey (the task's first ask)

[D crates/nf-game/src/mission/c07000011.rs; L ee_start_05.bin @0x2a4188] — level 07000011 (part 0
of mission 0x07000011) owns exactly 2 objective records:

- **0x04000035 "Escape from the tower"** (ch 0x64 = 100, enable 0);
- **0x04000036 "Gain access to lower office floor"** (ch 0x48, enable 0).

**Before this session, NEITHER was asserted anywhere**: `walkreal_07000011_live.rs` walked the
whole route but asserted only channels (0x7E spawn touch, no 0x62 death, no 0x7C atrium plates)
and the exit; `walkreal_07000011_route_live.rs` the same; `walk_07000012_live.rs` already asserted
the 0x48 completion on 07000012's side of the exit. No objective-state assertion existed for
07000011 itself. Both legs are now added.

## What landed (commit eb399e40, one file: `crates/nf-view/tests/walkreal_07000011_live.rs`)

1. **Both records asserted ANNOUNCED at load** — `mission.states[15] == [3, 3]` after the first
   monitor ticks [M `Mission_Init` 0x182270 + `Mission_MonitorObjectives` 0x1827d8:41-54: both
   records' enable is 0 → state 0 → the case-0 arm announces on the level].
2. **The negatives on the level** — at the stairwell bottom AND at the exit: ch 0x48 off, ch 100
   off, both records still 3. Evidence: **no writer for ch 0x48 exists on 07000011** [D
   docs/levels/07000011.md §2 — every trigger out, every script stream, every door/pickup/drone
   param enumerated], and ch 100 is the monitor's own AND (`bVar9` over every record's condition,
   decompiled 0x1827d8) which cannot fire while any part's condition channel is still 0 — its
   first fire is on 07000013 [D §2].
3. **"Gain access to lower office floor" asserted COMPLETE (state 4) through the real chain**:
   the walk ends on the exit firing to 07000012 [M Trigger_Activate 0x1b2318 →
   `ResetMap_LevelToLoad` 0x1b24b8], then loads 07000012 — whose spawn #0 (22.49, 0.19, 19.16
   [D]) stands inside TouchOnce **#679**'s tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62) [D] —
   ch 0x48 fires under Bond's feet and the monitor completes the record [M 0x1827d8 case 3 →
   state 4; the same writer the already-green `walk_07000012_live` pins]. "Escape from the tower"
   asserts still announced there (its writer is the whole-mission AND on 07000013).
4. **THE OLD 2 m POCKET STAND-IN IS GONE.** The stairwell door **#192** IS the parapet-pocket
   crossing: the disc puts the E `door_large_stairwell` (10.22, 46.75, −5.21 [D], lock ch 0x12)
   IN the W1/W2 parapet wall line — the doorway is the opening (the parapets are continuous and
   the game's own nav graph has no link across the pocket either [D nav BFS: the office-floor
   component ends at n45/n46/n47, the stair chain starts fresh at n94]). The walk now goes
   balcony → the door's front tile (walked, +6 ticks) → **E-open** (the keypad lock #177 is
   create-unlocked on Operative [D p6 = 1; M 0x1ae1f8:67-69; Lock_Update case 1 0x1ae3d8 holds
   ch 0x12]) → through the doorway (+39 ticks) → the flights. The `bond.pos` write is removed;
   the leg ran with **0 stuck, 0 falls**. The prior session's open live-check request
   (the pocket crossing) is closed by this route.

## Verification (what ran)

- `walkreal_07000011_live` (the edited walk, fresh builds of the whole 4-crate test set):
  **1 passed / 0 failed, 5.86 s**, 3262 ticks start-to-finish, stuck 0, falls 0.
- Suite slices on the pre-merge tree (`tools/test/suite.py --only`, `--test-threads=1`, slim ISO —
  the only allowed reds there are the 3 `disc_golden` tests, none in these slices):
  - **units + the 07000011 family + 07000012 walkers** (nf_game, nf_formats, nf_audio, nf_view,
    walk/walkreal 07000011 + route, interactables, rope_guard_hitbox, stairwell_* ×5,
    walk/walkreal 07000012): **`836 passed, 0 failed, 8 ignored  (4 min)`**
  - **the mission-chain neighbours** (walk/walkreal 07000013, walk/walkreal 0700004a, satchel,
    level_carry): **`15 passed, 0 failed, 0 ignored  (0 min)`**
  - **the transition flows** (campaign_flow, gameflow_stack_sp_run, mission_stats_carry,
    level_end, hold_npcs): **`14 passed, 1 failed, 0 ignored  (7 min)`** — the 1 is
    `TIMEOUT campaign_flow_live after 440 s` (the slice's own `--limit` killed it mid-run at
    `front_end_chain_01_to_05`), not an assertion failure; the binary needs >7 min and did not
    fit the window. This tree's diff is one leaf test file, which campaign_flow does not share.
- **The final merged tree re-run** (units nf_game + nf_view, the whole 07000011/12/13/4a walkreal
  family, level_carry): **`720 passed, 0 failed, 5 ignored  (2 min)`** — zero FAILED/TIMEOUT
  lines in this slice's log.
- The full 885-binary suite: `tools/test/suite.py` daemonised (`setsid nohup`) was killed by the
  sandbox between tool calls twice (21:37, 21:40 — the second kill left `suite11.out` empty), and
  the ~12 min build plus the ~3 h run do not fit this 2-hour window, so the slices above stand in
  for the total line (the level-09 precedent). The untouched tail is the next session's first run.

## What's left

1. The full-suite tail (~885 binaries minus the slices above) on the merged tree.
2. `campaign_flow_live` re-run with a larger limit (limit-timeout only).
3. "Escape from the tower" (ch 0x64) completes on 07000013, where the AND finally fires — the
   every-objective chain can be extended there next (07000012 → 0700004a → 07000013 legs, ending
   on the mission-complete hand-off [M Mission_Update 0x182c60 case 6]), if the series wants the
   ch-0x64 completion pinned end-to-end on real input.

## Live checks for the main agent

None new. (The previous session's pocket-crossing live check is RESOLVED: the crossing is the
stairwell door #192's doorway, now walked — see above. The level's known draw gaps stay as listed
in docs/levels/07000011-audit.md for the main agent.)

## Usage

Not visible to me: this environment shows no token/credit/context-window readout for the session.
(The only numbers I can read are the per-tool-call durations; no token or cost figures exist in
any output I received.)
