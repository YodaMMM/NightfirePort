## Your task: `level-16-finale-shortcuts`, 16: the last test shortcuts after the finale cinema

origin/s/fix-16-finale made `walk_the_chasers_defeat_completes_the_mission` pass on real input (the pit blast-room shelter). Two older tests, `fix_finale_live` and `kiko_button_real_walk_live`, still move Bond to the upper floor after the second cinema (0x06000632) as a shortcut, and the latter's header wrongly says nothing on the disc clears the button lock (the clear is at t1056 in the 07900016 pack). Replace the moves with what the cinema does (it drops Bond beside the button at t1057 [D]) or real input, fix the header, and keep all 16 tests green. Continue from origin/s/fix-16-finale (merge origin/master first). Run walkreal_07000016_live, interactables_07000016_live, fix_finale_live, kiko_button_real_walk_live, blast_room_shelter_16_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-finale-shortcuts`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T22:48Z

---
## Report (done, 2026-11-01T00:23Z)


# level-16-finale-shortcuts — the report

Branch `s/level-16-finale-shortcuts` (origin/s/fix-16-finale ad5dd6a5 + origin/master 81da6c47, merged at 62ac8914). Work commit: 23609346 (+ this report).

## What the task was

`fix_finale_live` and `kiko_button_real_walk_live` moved Bond to the upper floor (2.5, 1.2, −153.9) right after the pack-cinema hand-back as a test-only shortcut, and kiko_button's header claimed nothing on the disc clears the button lock (ch 0x8d). Replace the moves with what the cinema does (the lift-out pack cinema 0x06000632 drops Bond beside the button at t1057 [D]) or real input, fix the header, keep all 16 tests green.

## What was done

1. **fix_finale_live** (`the_07000016_kiko_death_button_defeat_completes_the_final_mission`): the upper-floor re-seat, the "nearest non-chaser" grenade sister, and the per-press re-seat at the button are GONE. The test keeps its documented telescope (fresh load → the silo plate #253, ch 0x8f, fire ch 0x81) and from there plays the disc's own chain on Input and fire: the pack cinema 0x06000631 hands Bond back on the pit floor (stream 8's single key [D pack_cinema_16_live; M Player_Enable 0x195498]) → the west blast room shelters both ch-0xf3 pulses (559/510-tick windows asserted [D 0600090a streams 6/9]) → the chaser falls to the test's grenade (her death raises nothing, param 12 = 0 [D #261]) → the pit squad falls to Bond's fire (a squad death never raises ch 0x90 — the sister, folded into the first kill) → ch 0xbf [D mux #611] → **the lift-out pack cinema 0x06000632 (07900016.bin)** plays no-skip and the test asserts its t1057 hand-back at key 1868 = (5.39, 1.34, −158.31) beside the button [D] and the gate off → Bond walks the last metres (the west lane) and his own E fires ch 0x90 → mux #618 → ch 100 → MissionState 6 → 0x7300017 [M 0x182c60:111-112]. No `bond.pos` writes remain after the silo-plate telescope.
2. **kiko_button_real_walk_live** (`the_07000016_real_walk_reaches_the_kiko_death_button_and_the_cinema_clears_the_gate`): the walked route (proving the corridor touch #616 arms ch 0x8d [D]) now continues through the same disc chain — shelter pulses, grenade chaser, fire squad, no pos writes — and the **lift-out cinema's t1056 clear is asserted on the walked route**: ch 0x8d goes OFF with the hand-back at key 1868 [D 07900016.bin]. The walked E then COMPLETES the mission (the old "the walked press is INHIBITED" pin is gone — it rested on the wrong no-clear claim). The gate-off sister (fresh load, teleport reach) is unchanged.
3. **The header fix**: kiko_button's header now states the disc DOES clear the lock — the old census decoded 07000016.bin + 07800016.bin and missed the **07900016.bin pack**, which holds the lift-out cinema 0x06000632: stream 16 writes ch 141 = 0x8d = 0 at t1056, stream 13's event 6 at t1057 hands Bond back at key 1868 [D; M Script_EventHandler 0x1d94c0 event 6 → Script_Free → Player_Enable 0x195498]. The docs (docs/levels/07000016.md, docs/systems/hurt-volumes.md) already carried the correct chain (fix-16-finale updated them); only the test header was stale.
4. One timing note: Bond's gun duel with the chaser from inside the shelter ran ~1100 ticks on walkreal's route but ~4000 on these routes, missing the 6310 door deadline [D leaves shut 6310..6660]; both tests therefore kill the chaser with the test's own grenade at her spawn point (the fuse-only explosion [M Bullet_CollisionHandler 0x12ce80]) — the shape both tests used before this task. The pit squad dies to Bond's fire (walkreal shape, no deadline after pulse 2).

## How it was verified

- The five named binaries on the final tree (`cargo test --offline --release --test-threads=1`):
  - `walkreal_07000016_live` **9 passed / 0 failed / 0 ignored**
  - `interactables_07000016_live` **8 passed / 0 failed / 0 ignored**
  - `fix_finale_live` **4 passed / 0 failed / 0 ignored**
  - `kiko_button_real_walk_live` **1 passed / 0 failed / 0 ignored**
  - `blast_room_shelter_16_live` **2 passed / 0 failed / 0 ignored**
  The 16 finale-chain tests (walkreal 9 + fix_finale 4 + blast_room 2 + kiko 1) are all green.
- `nf_game` **588 passed / 0 failed / 5 ignored**.
- Full suite (tools/test/chunk_par_tmp.py — this sandbox kills true background jobs, so the chunked foreground 2-worker runner with a fresh state file; priorities: the 07000016 family + nf_game): **394/996 binaries recorded by the 00:22 UTC stop line = 1782 passed / 1 failed / 122 ignored**. The one failure is `probe_pylon_render` (rc 101): the wgpu adapter enumeration panic — this sandbox has no GPU (the same environment limit previous tasks reported; `fix_0c_pylon_live` and the rest of the tail were not reached). The 07000016 family inside the suite: fix_finale_live 4/0/0, kiko_button_real_walk_live 1/0/0, blast_room_shelter_16_live 2/0/0, walkreal_07000016_live 9/0/0, interactables_07000016_live 8/0/0, walk_07000016_live 3/0/0, pack_cinema_16_live 1/0/0, census_07000016 0/0/3 (ignored-by-design; its dump is opt-in).

## What's left

- The suite tail (~600 binaries: the remaining nf-view walkreal/interactables families and the long GPU probes) was not reached by the stop line — the state file records 394 with zero real failures; the next message can resume from a fresh run.
- The open live check below.

## Live checks for the main agent

- Whether the GAME's contact pass registers touch #616 for the walking player at all (the port's Trigger_Hit is collision-tri-based, the game's is glist-bounds-based): fresh-boot PCSX2, walk z −144..−147 in the mid-corridor of 07000016, watch `switch_channels[141]` (0x26fc90+141 = 0x26fd1d): 0→1 on the crossing, then 1→0 when the lift-out cinema runs (its t1056 clear), confirms arm AND clear on the shipped game; a steady 0 means the game's #616 bounds differ from the port's tris and the walked press would complete without the cinema.

## Usage

not visible to me
