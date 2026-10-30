# level-0b-spawn-to-end-walk — the fresh-spawn walk to the mission's end

## What's done

Branch `s/level-0b-spawn-to-end-walk` = origin/master 814c8976 + the walkthrough refactor.

**`walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end` (new)** — a FRESH load of 0b
(`load_with_carry(..., None)`, no channel pokes anywhere) walked to the mission's end on real
input, reusing the existing legs' routes. The walkthrough body is now shared
(`walk_to_the_mission_end(w, fresh)`); the warm test `walk_balcony_to_the_roof_jump` keeps its
load and becomes the `fresh = false` arm — its tick sequence is unchanged (3/0 green).

**What the fresh boot proved (all cited in the test):**
- The carried objectives' channels (records 1-5: ch 0x61 inverted + 0x46-0x49
  [L ee_snapshot 0x2a3f78..0x2a407f, init values 0]) read OFF at the fresh boot — asserted:
  `Mission_Init` 0x182270 wipes only the current + future parts' condition channels to their
  init values and leaves previous parts alone (the part<order arm only reads the count,
  disasm 0x182488-0x1824f8), and their writers live on 09/0a — out of a fresh 0b load's reach.
  The monitored AND [M 0x1827d8:119-123] therefore cannot fold on a fresh boot — the mission
  end rides **OUT #304's OWN ch 100 write** (the mirror of ch 0x41 into ch 0x5A AND ch 0x64
  [D 0700000b.md §objectives]), which Mission_Update latches from any writer [M 0x182c60:23-28].
  The fresh walk asserts ch 0x5A + ch 100 + MissionState 6 + to_end 0x0730000b.
- **Objective 9's real writer found and used** (the task's port-gap hunt): the terminal script
  `06000035` stream 8 spawns at its tick 0x1e5 the type-0xEA TouchOnce entity (hash 0x2000ab5,
  params [6, 4, ..] = out ch 6, mask 4 = use [D nis dump; M `Script_CreateEntity` 0x1d8968
  case 0xEA → `Trigger_TouchOnce`]). The port's per-tick script-trigger rebuild
  (drone.rs, the control-list head) already materializes it — the fresh walk walks to the
  spawned plate (centre (−1.15, 48.36, −13.89), box (−1.73,47.92,−14.20)..(−0.57,48.81,−13.57))
  and E-fires it: **ch 0x06 → OR #295 → ch 0x4E (objective 9)** with NO drive. The warm walk
  keeps the documented ch 0x08 drive (the disc's ch 0x54/0x06 writer stays the level's one open
  channel chain); on the fresh walk nothing is driven.
- The ride: the cab boarding worked this run (boarded → rode true, no stand-in), the lock's
  walked E raised ch 9 [M 0x1b02a8], the Q-Worm install landed [M 0x1aee20] (the boot loadout
  owns 0x58 [D weapon-matrix], no grant needed), the roof cut-scene re-placed Bond on the deck,
  the parachute touch + the deck jump fired the SEQ. Fresh run total: **37886 ticks**.

## How it was verified

- `walkreal_0700000b_live` (nf-view, NF_ISO set, --test-threads=1): **3 passed; 0 failed**
  (`finished in 41.86s`) — the warm walkthrough, the fresh walk, the sister roof-cut-scene test.
- The full suite: this sandbox kills background jobs between tool calls (the -3 report hit the
  same), so the run went through the chunked runner (`tools/test/chunk_suite.py`, state
  `../suite0b_state.json`): **61/916 binaries recorded, 287 passed / 0 failed / 3 ignored**
  before the window ended. One rc≠0 with 0 failed: `mp_online_failures` (a network-bind binary
  in the slim sandbox). The build pass completed for all four crates (916 binaries) and the
  state file accumulates — the next window continues it.
- What ran beyond the touched binary: the chunked binaries above (nf-game/nf-formats/nf-audio/nf-view
  mix) plus the build of all 916.

## What's left

1. The FULL-suite total line on this tree (continue `python3 tools/test/chunk_suite.py
   ../suite0b_state.json --budget 420` until ALL DONE; the build is cached).
2. The warm walk's desk leg still ends in the documented stand-in (the n25/door-#5 wedge —
   the -3 report's item 2, unchanged this round).
3. The ch 0x54/0x06 open item: the fresh walk proves ch 0x06's spawned-plate writer is REAL;
   the OR mux's other input (ch 0x08 ← OUT #294 ← 0x54) stays open exactly as documented.

## Live checks for the main agent

- None new. The spawned use-plate's E is test-verified in the port; a PCSX2 confirm (fresh-boot
  0b, after the keycard chain, E at the terminal console → "Retrieve the NightFire files"
  completes) would close the audit's §6.1 question from the game side.

## Usage

not visible to me (no token/credit/context-window figures are shown in this environment; the
sandbox shows only the shell).
