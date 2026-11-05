# playscript-deaf-0x168 — scripted guards on 0x6000168 ignore gunfire while their script plays

Branch `s/playscript-deaf-0x168` (cut from origin/master a89818235; origin/master fully merged into it —
`git rev-list --count HEAD..origin/master` = 0). Three commits: 1ca68313e (cut + WORKLOG), e7c3c595f (the fix),
70dfa5dbd (the census + the unit pin through the real ENTER).

## What's done

**1. The fix (e7c3c595f).** The port's `(PLAY_SCRIPT, msg::SOUND)` arm in `crates/nf-game/src/drone.rs`
(now line 7395) held the deaf window only for start scripts 0x6000169 and 0x60007ac; a guard playing
0x6000168 broke out of the script on gunfire. Fixed: the condition now covers all three scripts.

Evidence, re-verified this session from `work/decomp/ACTION.ELF/0015ba68_NDrone2_DSTATE_PlayScript__FP10DCVars_tagP9Drone_tagP7obj_tagP9MsgObject.c` and the raw ELF bytes of `work/elf/ACTION.ELF`:

- The 0x6000168 and 0x6000169 ENTER arms fall into a SHARED tail: `DroneAnim_SetScript(+0x554 …)` →
  `NDrone2_AttackTalk` → `*(drone+0x4F8) |= 0x400000` [M decomp line 73]. Disassembly confirms the OR at the
  machine level: `lw r4,4(r16); lui r3,0x0040; lw r2,0x4f8(r4); or r2,r2,r3 (0x00431025); sw r2,0x4f8(r4)`
  at 0x15bbf4–0x15bc08 [M].
- The 0x60007ac ENTER arm sets the same bit by its OWN OR right after its SetScript [M decomp line 84;
  disasm `lui r5,0x0040; lw r2,0x4f8(r3); or r2,r2,r5 (0x00451025); sw r2,0x4f8(r3)` at 0x15bc40–0x15bc54].
- The default / 0x60005e6 / 0x60008ae arms return after SetScript with NO OR [M 0x15bb50–0x15bba4] —
  only the three scripts above are deaf.
- `DroneVision_AlertSound` 0x177800 returns 0 while the bit stands [M :19–21: `if (prop(0x1f) == 0 ||
  (+0x4F8 & 0x400000) != 0)` → return 0], so those scripted guards are DEAF to gunfire while the script plays.
- The window ends at the script's natural end (msg 5): all three arms clear the bit there —
  `&= 0xffbfffff` at decomp lines 128 (0x6000169), 135 (0x60007ac), 153 (0x6000168 shared) [M], all inside
  case 5. The port zeroes `start_script` unconditionally in its `(PLAY_SCRIPT, msg::ANIM_DONE)` arm
  (drone.rs 7367) and only then branches per script (0x168/0x169 → CallAnim + AttackTalk [M :149–157];
  others → start state) — the deaf window therefore ends on the same message for all three scripts.
- The comment that cited ":36-44" is fixed: that block is the 0x60005e6/0x60008ae arm, which has NO OR;
  the citation now points at :70-73/:84 with the disasm addresses.
- docs/reports/audit-voice-triggers.md row 4's "possibly inverted" reading is settled: both arms SET the
  bit (decompile + disasm above); 0x6000168 was simply missing from the port's condition. (That report file
  is not on this branch's tree; the reading is quoted from the task.)

**2. The disc census [D] (70dfa5dbd).** `crates/nf-formats/tests/drone_start_script_census.rs` walks every
`070*.BIN` level on the real disc, takes each main map's kind-0x0F (drone) placements, and resolves them
through the port's own `drone::start_script` (placement parameter 1 custom, else the `NDrone2_DefaultInit`
0x14b300 table row — the only +0x554 writer [M 0x14b300:611-627]). Result on the retail disc:

- **NO placement anywhere resolves to 0x06000168 — the arm (and its deafness) is dead on retail data.**
  So, per the task: there is no disc guard to pin live on 0x6000168; the unit pin below is the pin.
- Exactly two placements resolve to the sibling deaf scripts: **07000001.BIN → 0x06000169** (the drive
  guard, docs/systems/drone-first-shot.md §1) and **07000012.BIN → 0x060007ac**. No other 07* level has
  any deaf-script placement.

**3. The unit pin (70dfa5dbd, in drone.rs's test module).**
`play_script_0x168_is_deaf_to_gunfire_until_the_script_ends` drives the real state ENTER (which runs
`play_script` → `anim.script = Some(0x6000168)` [M 0x15ba68:49-53/70-73]) and pins, in order:

- gunfire (msg 0x14) while the script plays → NO state transition, the script holds [M 0x177800:19-21];
- msg 5 (ANIM_DONE) → `start_script` clears [M :153] and the state HOLDS (CallAnim + AttackTalk, no
  SetState) [M :149-157];
- the NEXT gunshot → mode 9 at alert 1.0 is AlertSound's attack row [M :51-68] → transition into ATTACK:
  the guard HEARS once the script ends.
- The commit message also records the pre-fix red run: before the fix the same sequence left the script on
  the shot (state 19 via ATTACK enter) — the user's symptom, reproduced then fixed.

Determinism untouched: no RNG draws added (the pin uses the shared `Rng::new(0x5eed)`), fixed 60 Hz, no
wall clock.

## How it was verified

Directly required binaries (each whole, `--test-threads=1`, NF_ISO set):

| binary | result |
|---|---|
| nf_game (lib, incl. the unit pin) | 667 passed, 0 failed, 5 ignored |
| nf_formats (lib) | 92 passed, 0 failed |
| nf_audio (lib) | 23 passed, 0 failed |
| nf_view (lib) | 100 passed, 0 failed |
| drone_start_script_census [D pin, real disc] | 1 passed, 0 failed |
| cover_nodes_live | 3 passed, 0 failed |
| fix_06_alarm_live | 2 passed, 0 failed |
| ai_live | 37 passed, 0 failed |
| ai_missing_states_live | 9 passed, 0 failed |
| ai_points_live | 4 passed, 0 failed |
| ai_traces_live | 4 passed, 0 failed |
| driving_ai_follow | 9 passed, 0 failed |
| driving_ai_physics | 8 passed, 0 failed |
| fix_ai_remainder_live (nf-game + nf-view copies) | 2 passed, 0 failed |
| guard_ai_other4_live | 4 passed, 0 failed |
| stealth_ai_live | 9 passed, 0 failed |
| stun_ai_recover_live | 2 passed, 0 failed |
| npc_doors_live | 6 passed, 0 failed, 1 ignored |
| fix_guard_misc_live | 6 passed, 0 failed |
| fix_guard_weapons_live | 3 passed, 0 failed |
| guard_first_shot_01_live | 2 passed, 0 failed |
| corridor_guards_07000012_live | 1 passed, 0 failed |
| start_corridor_guard_07000011_live | 8 passed, 0 failed |
| abseil_elevator_live | 3 passed, 0 failed |
| nis_coder_break_live | 3 passed, 0 failed |
| vision_bit_census | 1 passed, 0 failed |

The last nine are the test binaries that grep for `start_script|PLAY_SCRIPT` plus the door/guard set
required by the merge lessons (drone.rs touched). All green.

**Full suite (tools/test/suite.py logic, run per-binary by a resumable foreground runner because background
processes and long tool calls are killed in this sandbox): 2955 passed, 9 failed, 106 ignored across 631 of
1114 test binaries.** The 9 failures are all environmental, none code-related:

- `disc_golden`: bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest,
  directory_tree_matches_the_golden_manifest — the 3 disc_golden tests the driver allows to fail on the
  slim ISO.
- fix_0c_pylon_live (2 tests), glass_guards_14_live (1), missing_scenery_live (2) — all panic at their
  `wgpu::request_adapter(...).expect("adapter")` with `NotFound` — this sandbox has no GPU (the merge
  lessons' known limitation; these are pixel-comparison render tests).
- identity_disc::the_real_disc_passes_the_identity_check — expects the full retail image (1,191,072
  sectors = 2,439,315,456 bytes); the sandbox has the onfoot cut (1,061,754,880 bytes). The branch does not
  touch `crates/nf-formats/src/` at all (`git diff origin/master..HEAD --stat`: only
  nf-formats/tests/drone_start_script_census.rs added, drone.rs, WORKLOG), and the same failure appears in
  the previous session's suite log on the pre-fix tree.
- mp_online_failures was SIGKILLed in-suite (exit -9) and its `join_in_progress_…_reconnect` test hangs
  past 170 s standalone (twice) — real-socket reconnect behaviour the sandbox cannot complete; not counted
  in the total above, unrelated to the branch.

The remaining ~483 binaries were not reached (the gateway kept cancelling long foreground calls; the
resumable runner persisted state and I spent the window on the binaries that matter for this task — every
binary named by the task and every binary whose tests grep the touched code ran whole and green). No
binaries failed that use the code this branch changed.

## What's left

- The other ~483 suite binaries (nothing branch-related is expected among them — the four crate lib
  binaries and every touched-code binary are green).
- Optional: teach audit-voice-triggers.md row 4 the settled reading (that doc is not on this branch's tree;
  it belongs to that task's branch).

## Live checks for the main agent

No disc guard plays 0x6000168 (census above), so the live pin lands on the sibling scripts:

1. **07000001 drive guard (0x06000169)** — load the level, let his start script begin, fire a shot near
   him while it plays. Expected: he stays in PlayScript (no combat transition) while the script plays;
   gdbwatch on his drone record +0x4F8 should show the ENTER write `|= 0x400000` (the OR at 0x15bbf4–
   0x15bc08) and the clear `&= 0xffbfffff` at the script's msg 5 (case 5, :128). After msg 5 the next shot
   should pull him to combat (AlertSound 0x177800 mode-9/0x16 rows).
2. **07000012 corridor guard (0x06007ac)** — same drill; the ENTER OR is its own at 0x15bc40–0x15bc54.
3. If any build ever places a 0x6000168 guard (custom level/param 1 = 0x168): same expected behaviour —
   deaf while scripted, hears at msg 5 + next shot. Not reachable on retail data.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures anywhere I can
read (no banner, no status line, no API surface exposed to the worker).
