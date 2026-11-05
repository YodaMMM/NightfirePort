# death-nis-music-restart

The death music now also plays when a death cut-scene starts from the SP_Update case-1 restart arm,
closing the gap the subtitle-flush-camera-scripts task left in a comment on
`crates/nf-game/src/nis.rs`.

## What the game does (the evidence)

`Script_Play` 0x1db588 ends with the camera tail: on a camera script (`+0xAA8 & 0x40` — the port's
flags bit 0x40 [M 0x1d8248 → Script::load]) whose hash is one of the three death NISs
(`Script_IsDeathNIS` 0x1dae60: 0x6000952 / 0x600098a / 0x6000991) it posts the death music jump
`Music_Event(6, 1)` [M decompile 001db588 + disasm 0x1db608-0x1db610], then `Text_FlushAllSubtitles`
runs UN-gated for every camera script [M `jal` at 0x1db618]. The tail lives inside `Script_Play`, so
it runs wherever `Script_Play` runs:

- the load arm — `SP_LoadScript` 0x1dbc08 plays every mode arm through `Script_Play` [M 001dbc08];
- the restart arm — `SP_Update` 0x1dbda8 case 1 reaches `LAB_001dc174: Script_Play(si, mode == 8)`
  [M 001dbda8 case 1]. This is the arm the port left flush-only.

## The fix

Both port sites now share one helper, `Script::push_camera_tail` (nis.rs): the death-NIS
`Request::Event { kind: 0x41 }` (drained onto the board by `Drones::drain_script_requests`; the
board's event-6 consumer is the on-death marker jump — the same jump `Player_CheckForDeath` posts),
then `Request::TextFlush`. The case-1 arm calls it after `s.start(looping)` exactly as the game's
case 1 does (`state = 2`, `countdown = 5`, `Script_Play(si, mode == 8)`); the load arm keeps its
flush-before-first-line order (the tail lands before SP_LoadScript's mode arm reaches the script's
own first `Script_Update` [M 0x1dbc08]). The gap comment on the case-1 arm is removed.

`Script::start` pushes no requests, so the helper's position relative to `s.start` does not reorder
the outbox; inside the helper the order is the game's (`Music_Event` 0x1db610 before
`Text_FlushAllSubtitles` 0x1db618), and the case-1 pin asserts that order.

## Which placements can start a death NIS through the case-1 arm [D]

A disc scan over every `0700*.bin` archive in FILES.BIN (all 36, wrappers and packs included) found
exactly five 0xD9 ScriptPlayer placements carrying a death-NIS hash, all as script A (script B 0 in
every record):

| archive | SP | script | mode A/B | stay | mask | out |
| --- | --- | --- | --- | --- | --- | --- |
| 07000005.bin | #284 | 0x6000952 | 0 / 0 | 0 | 0x1 | 0xbe |
| 07000008.bin | #307 | 0x6000991 | 0 / 0 | 0 | 0x1 | 0x7c |
| 0700000b.bin | #325 | 0x600098a | 0 / 0 | 1 | 0x1 | 0x7b |
| 07000011.bin | #313 | 0x600098a | 0 / 0 | 0 | 0x1 | 0x7c |
| 07000012.bin | #767 | 0x600098a | 0 / 0 | 0 | 0x1 | 0x7c |

(07000012 #767 is new to the docs — the level audits had listed only the other four.)

None of them can reach case 1 in real play:

- mode A 0 and mask 0x1 → they create into state 0 (`SP_CreateScriptPlayer` 0x1db9a8; mode 0 ∉
  {0,1} priming does not apply, mask ≠ 0 → state 0 [M 0x1db9a8:74-86]), so their FIRST start is
  SP_Update case 0 — whose `SP_LoadScript` runs the tail through the load arm (already ported).
- After the scene finishes, case 2's end moves them to state 3 (stay 0) or state 5 (stay 1, #325
  deletes). Case 1 additionally requires a pass through case 3 (a re-triggered restart, state 3 →
  4) and another script end (state 4 → 1) — the death NIS would have to run to completion TWICE
  and be triggered a THIRD time.
- Every replay of a death NIS re-runs its own mid-scene ch-0x62 write (e.g. 07000008's 0x06000991
  writes ch 0x62 at t89 [D the script decode]), which fails the level [M Mission_Update 0x182c60
  case 1 → MissionState 3] — the flow ends the level on the first run, so no placement walks that
  path.

**So: no real-path pin exists; the pin is a unit pin** (`the_case1_restart_of_a_death_nis_runs_the_same_camera_tail`
in nis.rs), driving a primed death-NIS SP through `Player::update` the way the game's case 1 would
(mask 1, the out channel on) and asserting the kind-0x41 Event + TextFlush in the game's order,
plus the sister (a non-death camera NIS flushes without the jump).

## Observed, not tasked (left as-is, listed for the driver)

- `SP_Update` case 3 jumps to the same `LAB_001dc174` [M 001dbda8 case 3], so a state-3 retrigger
  restart also runs the tail; the port's case-3 arm pushes no tail either. Same unreachability
  class as case 1 for a death NIS (state 3 needs one completed death-NIS run), and the task scoped
  case 1, so it is left untouched.
- The game's case 0 calls `SP_LoadScript` (which `Script_Play`s internally) and then `Script_Play`
  again explicitly [M 0x1dbda8:105-107] — the tail runs twice on a case-0 start. The port runs it
  once (through `load`). `Music_Event(6,1)` sets `EventList[6]` (idempotent) and the flush empties
  (idempotent), so the once is behaviourally equal.
- The two zero-script 0xD9 placements on wrapper 07000041 idle forever with mask 0 [M
  script-player.md] and can start nothing.
- The census is airtight for the disc's placements: a second scan pass checked EVERY param of
  EVERY placement kind (not only 0xD9) in all 36 archives for a death-NIS hash and found the SAME
  five records, param 0 only — no switch/fusebox/useobj or other embedded-SP carrier exists [D].
  The code-created SPs (`Switch_Create` 0x1adcc0, the fusebox's mode-0x40 preload) take their
  scripts from those params, so no embedded SP can prime a death NIS into state 1 either.

## Verification

- `nf-game` lib: 677 passed, 2 failed — the 2 are the PRE-EXISTING master reds
  `mp::bots::fell_out_tests::fell_out_map_commits_bot_idle_on_the_next_tick_and_spares_other_bots`
  and `..._forces_farthest_spawn_and_wipes_the_goal_machine` (subtract overflow at bots.rs:1368;
  both fail with the branch's changes stashed, i.e. on master 2f0c88e2c itself). All 14
  `nis::tests` pass, including the new case-1 pin and the load-arm pin.
- The music binaries (`menu_music_flow`, `music_triggers_live`, `music_unported_consumers_live`),
  the nis binaries (`nis_coder_break_live`, `nis_event9_live`, `nis_fade_lifecycle_live`,
  `nis_page_boot_live`) and the flush pins (`subtitles_live`, `level06_start_subtitles_live`) run
  inside the full suite — per-binary pass counts below.
- Full suite (`tools/test/suite.py`, slim ISO): see `## Suite`.

## Suite

(to be filled from the final suite run)

## Left

Nothing on the tasked scope. The case-3 tail and the case-0 double-play are listed above as
observed deviations, both unreachable for a death NIS in real play.

## Live checks for the main agent

None needed — no real path reaches case 1 with a death NIS, so there is nothing observable in a
live run beyond what the existing death-NIS pins already cover.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage banner in any tool output), so there are no numbers to copy.
