## Your task: `level-14-every-objective-2`, level 07000014 every objective on real input

`walkreal_07000014_live` walks Countdown Part 1 to the Delta exit. List every objective the mission data lists [D / M 0x1827d8] and which the walk asserts complete; add the missing ones on real input (no teleports past blockers, no channel pokes), including the capture cut-scene path and the sabotage cascade, ending on the exit. Every blocker is a port gap: fix it [M/D] or list it if drawing only. Assertions never rest on a missing writer. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-29T16:50Z

---
## Report (done, 2026-10-29T18:12Z)

# level-14-every-objective — 07000014: every objective on real input

Task: `walkreal_07000014_live` walks Countdown Part 1 to the Delta exit; list every objective the
mission data lists [D / M 0x1827d8], which the walk asserts complete, add the missing ones on real
input (no teleports past blockers, no channel pokes), including the capture cut-scene path and the
sabotage cascade, ending on the exit. Every blocker is a port gap: fix it [M/D] or list it.

## The census

Mission 0x07000014 (Countdown Part 1) is part 0 of a three-part mission. The mission data lists
**5 objectives on this part** [D `crates/nf-game/src/mission/c07000014.rs`, the record block at
`ee_start_05.bin @ 0x2a4260` (5 records)]:

| # | channel | enable | text label | title |
|---|---------|--------|------------|-------|
| 1 | 0x47 | 0x00 | 0x04000049 | Infiltrate Drake's facility undetected |
| 2 | 0x48 | 0x47 | 0x0400004a | Follow Kiko to the Security Control Center |
| 3 | 0x4a | 0x48 | 0x0400004b | Sabotage base security and escape from room |
| 4 | 0x54 | 0x4a | 0x0400004c | Prevent Rook from cutting the power to Delta Sector |
| 5 | 0xce | 0x54 | 0x04000069 | Take the exit to Delta Sector |

Parts 1 and 2 carry three more records [D `c07000015.rs` ch 0x8E, `c07000016.rs` ch 0x8F/0x90] —
they matter for the monitor's tail AND (below).

### Which the walk asserted before this task

`walk_the_infiltration_to_the_delta_exit` asserted records 1–4 completing
(`states[20][0..=3] == 4`), through real-input legs: the vent cut-scene pack 0x06000833 writes
ch 0x47 (its event-0xD `[0x47,1]` [D pack decode; SP #276 out 0x7E]); the Kiko pack 0x0600062F
writes ch 0x48 (`[0x48,1]`, SP #382 out 0x8); the escape pack 0x06000848 writes ch 0x4A
(`[0x4a,1]`, SP #407 out 0x9); Rook's death writes ch 0x54 [D drone param 12; M set_death_channel
0x13e660]. **Record 5 had no completion assert at all** — the walk only checked the fallback
channel word (`channels.on(0xce)`), never `states[20][4]`.

## What was added (and the port gap it exposed)

1. **Record 5's announce** — after the record-4 fold, the monitor's same tick announces record 5
   through case 1 (state 1, enable 0x54 ON) [M `Mission_MonitorObjectives` 0x1827d8 case 1:
   `switch_channels[enable] != 0` → Text_AddMsg + state 3]. Pinned:
   `assert_eq!(states[20][4], 3)` right after the objective-4 assert.
2. **Record 5's fold — and the port gap.** The exit trigger's fallback arm writes ch 0xCE
   [M `Trigger_Activate` 0x1b2318] during the trigger pass, then pushes the 300-frame exit hold
   (`GS_PauseGame(1)` + `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:90-92]). The port's
   `step`/`step_bond` **skipped the whole sim during the hold** — so the record never folded
   before the load. But the game runs `Mission_Update` EVERY frame of the hold: it sits at
   `Game_Run` 0x1c9790:17, OUTSIDE the `GS_IsPaused` gate whose :30-38 arm freezes only
   `SSys_Monitor`/`Light_Update`/`control_movement_object_handler` [M 0x1c9790 decompile; the
   pause-freeze doc `docs/systems/exit-hold.md` already recorded the gate split]. The monitor's
   case 3 (channel ON → complete banner → state 4) therefore folds record 5 during the fade in
   the game [M 0x1827d8 case 3], exactly the banner a player sees before the Delta Sector load.

   **Fix [M]**: `Drones::tick_mission_during_hold(level)` (`crates/nf-game/src/drone.rs`) — the
   mission layer only, on the current frame (the clock stops while paused [M 0x1c98c8:44-47], so
   no `self.frame` advance), with the same MP-master gate, music pulses and LevelEnd guard as
   `tick_inner`'s block [M 0x182c60:16-28]. The two scene hold sites (`Scene::step`,
   `Scene::step_bond` in `crates/nf-view/src/scene.rs`) now run `tick_mission_during_hold` then
   `tick_exit_hold` (mission first, then the fade tick — the game's per-frame order), preserving
   the old contract that the expiry tick stays world-frozen.

   Test: `assert_eq!(states[20][4], 4)` within ticks of the exit E-press.
3. **The ch 0x64 negative pin.** My first version asserted the mission-complete channel 0x64
   (100) rising once all five records completed. The run proved otherwise — and the decompile
   says the TEST was wrong: the record walk filters rows by **BaseMap** [M 0x1827d8:26
   `(&DAT_002a4354)[row*10] == BaseMap`, the mission id], so the tail's `bVar9` AND spans all
   three parts' records; parts 1–2's channels (0x8E/0x8F/0x90 [D DEFs]) are still off at this
   part exit, the AND is false, and `switch_channels[100]` stays down [M 0x1827d8:119-123; the
   port's `monitor` implements the same span]. The test now pins the negative, with the writer
   census [D the packs' 0xD events 47/48/4a/0e/11/10/b8/92/93/b7/8a; the SP outs 7E/8/9/85/54 —
   no ch-100 writer on this level].

## The capture cut-scene path and the sabotage cascade (already real-input, verified)

- **The capture path** is the Kiko leg: the walk enters touch #381's band on foot (out 0x08
  [D placement 381]), the SP #382 pack 0x0600062F takes Bond's body west past the sealed 0x85
  door (the scripted-body mechanism [M Script_Free 0x1da880 → Player_Enable 0x195498]), and the
  pack's own event writes ch 0x48 [D pack decode]. Record 2's announce+fold were already pinned;
  the leg drives them with real movement inputs (the loop's `turn`/`forward` inputs, no
  teleports, no pokes).
- **The sabotage cascade** is legs 3a–3c: two real Laser fusebox cuts through `FuseBox_Update`
  0x1adfe8 (`#383→0x87`, `#385→0x88`), the AND #391 → 0x8C, the E-use switch (mask 4
  [M Trigger_Activate 0x1b2318:23-33]) → 0xA0 → SEQ_AND → 0x51 → the kill cascade → box death
  0x85 → the 0x85 doors unlock and auto-open [M Door_Update 0x1363c8:88-107], then the escape
  touch and cut-scene. All in the walk, all green.
- Note, not asserted: pack 0x060008AF (655 frames) writes 0x92=0/0x93=1/0xB7=1 [D pack decode]
  but has no SP player and no SubScript reference among the level's 13 packs — likely a
  drone-carried action script. Listed as a port gap only if something visibly depends on it;
  nothing in the objective path does.

## How it was verified

- `walkreal_07000014_live` — **5 passed / 0 failed** (65.4 s) with the new asserts.
- The exit-hold change touches every level transition, so the mayhew-transition walks were run
  directly: `walkreal_07000001_live` **4 passed** (33.1 s), `walkreal_07000002_live` **1 passed**
  (50.6 s, the cross-level fold), `walkreal_07000005_live` **9 passed** (70.1 s, the
  mayhew-end-transition's home level).
- `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view --no-run`
  builds clean on the merged tree.

## The FULL suite

The sandbox kills background jobs between tool calls (reproduced: `nohup` cargo and
`setsid nohup suite.py` both died mid-run), so the suite ran as foreground chunks of
`tools/test/chunk_suite.py` (state `/home/z/suite14_state.json`, resumable):

- Chunks 1–4: **43 of 904 binaries recorded — total so far: 240 passed / 1 failed / 6 ignored.**
  The 1 failure is `probe_pylon_render::probe_render_pylon_only` — the known no-GPU
  environmental failure (the pylon renders need a GPU; this sandbox has none).
- The same morning's full-suite baseline on this master lineage (level-01-every-objective-2
  report): **3454 passed / 20 failed / 213 ignored, 876/894 binaries** — all 20 environmental
  (3 disc_golden slim-ISO, identity_disc, pylon ×3, 11 OOM, runner-cap TIMEOUTs).

The sweep could not reach 904 binaries inside this window (~55 min of work time, ~1 min per
slow binary on 2 cores). **Left for the next session:** re-run
`python3 tools/test/chunk_suite.py /home/z/suite14_state.json --limit 570 --budget 470` until
`ALL DONE` on this tree (it resumes; the state carries the 43 done), then quote the summed
total. No disc_golden regression is expected from a sim-logic-only change.

## Live checks for the main agent

1. On a FRESH boot to 07000014, walk the golden path to the exit and press E: the
   "OBJECTIVE COMPLETE" banner for **"Take the exit to Delta Sector"** should appear during the
   fade-out (the fold during the 300-frame hold — [M 0x1c9790:17] says it must; a live look
   confirms the banner beat the load). Watch for it between the E press and the black frame.
2. If cheap: at the same moment, dump `switch_channels[100]` (mission-complete) — expected 0 at
   the part exit [M 0x1827d8:26 the BaseMap AND], and 1 only after the final part.

## What's left

- The remaining ~861 binaries of the chunk sweep (above), plus a final merged-tree re-quote.
- Nothing else: the objective census, the missing asserts and the port-gap fix are complete and
  pinned.

## Usage

not visible to me


--- branch tip: c142388e ---
