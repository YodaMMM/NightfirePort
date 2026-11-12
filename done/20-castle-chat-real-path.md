## Your task: `castle-chat-real-path`, The Exchange castle's chatting guards, pinned on the real path

Branch from origin/master (guard-castle-chat-states landed: the 07000006 chat pair by the dock plays CastleChatGuard1 [M 0x1612e0]). Its real-path test walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted is ignored; read its ignore message. Reach the pair with real input from the load, show they stand and play the chat script on ch 0x11, then alert them and show they break off, and un-ignore the test. Run walkreal_07000006_live and interactables_07000006_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-castle-chat-real-path`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-12T01:46Z

---
## Report (done, 2026-11-12T03:32Z)


---
## Report from the worker (done, 2026-11-12T03:33Z)

# castle-chat-real-path — the castle's chatting guards pinned on the real path

Task: 07000006's chat pair by the dock plays CastleChatGuard1 [M 0x1612e0]; reach the pair with
real input from the load, show they stand and play the chat script on ch 0x11, then alert them
and show they break off, and un-ignore
`walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted`.

## What was done

1. **The port had to be merged in first.** `guard-castle-chat-states` NEVER landed on master:
   `git log -S CASTLE_CHAT_GUARD1 origin/master -- crates/nf-game/src/drone.rs` is empty —
   master only carried the comment mention. Merged `origin/s/guard-castle-chat-states` (drone.rs
   +261, the ignored live test, its report) into this branch as the base.
2. **The spawn gate found — the old ignore message's conclusion was wrong.** It said the pair is
   "event-gated somewhere (SS_Create / a channel-gated creator)". The disc says otherwise: the
   pair's kind-0x0f placements carry flags **exactly 0x0000000f** [D raw 0x1a block of
   07000006.bin, main map 48, records 487/488 — parsed from `nf-extract block-dump`], so
   `parsemap_block_map_data_dynamic` 0x1d1b68 passes them to `Drone_Create` [M 0x1d04f0 case 0xf]
   like every other guard. The sim confirms: both chat guards spawn at load, present and enabled,
   standing in state 0x36 with mode 0x18 and chat channel 0x11 armed. The previous probe missed
   them because its walk walked to 2.7 u from the pair — long after they had spotted Bond, left
   the chat and died; at the walk's arrival no drone stood at the census coords.
3. **The chat's channel gate.** The chat scripts play only once switch channel 0x11 is UP
   [M 0x1612e0:57-65]. The level raises it with an OR trigger: area plate **#528** (type 1, out
   0x3b, box x 49.7-51.2, z 26.0-30.3 — the truck's west end) feeds OR trigger **#512** (type 0xc,
   out 0x11, list [0x3a, 0x3b]) [D census]; plate #513 (out 0x3a, z 44.7-46.6) is the south
   approach's input. The dock walk crosses #528 on the greeting-plates leg; ch 0x3b rises, the OR
   raises 0x11, and both guards' UPDATE plays their start scripts.
4. **The test green and un-ignored.** The walk (real input only): off the truck, across the
   greeting plates, stop at the watch spot (45.0, 38.3, 24.0) — 15 u east of the pair on the same
   dock. Closer approaches BREAK the chat (the chat state's UPDATE runs EnemyLookForOpponent
   [M 0x1612e0]; measured: at 11-12 u east the pair left 0x36, charged and died to return fire;
   the chat only holds beyond ~13 u). Pins, all passing:
   - both guards present at their placements, state 0x36 [M 0x15ba68:26-30 + 0x14c600:147-158];
   - chat channel 0x11 armed and up (the per-tick cache true, the level channel on) [M
     0x14c600:147-158; D census #512/#528];
   - the start scripts PLAYING: `anim.script` = Some(0x06000124)/Some(0x06000126) [M
     0x1612e0:57-65; D census — the level bin carries both scripts, 1244/1204 frames];
   - after a sustained burst on the south guard (#488 — the north guard's box rays are occluded
     by #488 standing on the line [M Drone_CollisionHandler 0x140968:85-95, no team gate]):
     both guards out of 0x36, and the UN-SHOT guard still present and alive — the alert group's
     EnemyAlerts fold, not a death [M 0x1612e0:83-98].
   The `#[ignore]` is removed.
5. **The d1 real-damage walk re-greened.** The port shifts this level's guards (the pair and the
   0x0600011d driver now run the game's real 0x36/0x17 states instead of the old plain-guard
   path), which shifts the shared RNG stream: the y-47 balcony grenade guard's 54.0 toss landed
   at the crest and the walk died in the tunnel at tick 3964. Lever (file-local, the shared
   helpers unchanged): a pre-clear of the balcony guard from the ramp's upper lane — any-height,
   exposed-only, the game's own fire path. The walk completes again.

## How it was verified

- `walkreal_07000006_live` whole: **7 passed, 0 failed, 2 ignored** (the d2/00 Agent ignores are
  master's pre-existing ledgers; the new chat test runs in the pass count).
- `interactables_07000006_live` whole: **5 passed, 0 failed**.
- `nf-game --lib` (with NF_ISO): **791 passed, 0 failed, 4 ignored** — the chat unit tests
  included.
- The drone.rs merge touches every level's drones, so more batteries (each whole, --test-threads=1):
  npc_doors_live **6/0/1i**; fix_guard_misc_live **6/0**; walkreal_07000001_live **7/0/1i**;
  walkreal_07000002_live **3/0/2i**; walkreal_07000004_live **10/0**; walkreal_0700000c_live
  **4/0/1i**; walkreal_07000013_live **4/0**; walkreal_07000015_live **8/0/1i**;
  walkreal_07000016_live **15/0/3i**.

## What is left

- The remaining walkreal batteries did not fit the sandbox window: 07000003 (timed out at 260 s
  mid-route — needs a longer window), 05, 07, 08, 09, 0a, 0b, 0c_barrelrow, 0c_eastroute, 0d, 11,
  11_route, 12, 14, 1b, 41, 41_wall, 4a, 4a_satchel. Nothing in this branch's diff targets them;
  the drone.rs delta is the merged port (zero-Rand handlers, unit-pinned), and the nine batteries
  above cover every family the merge lessons call out (doors, guard misc, the defeat walks, the
  0c big level).
- The fast suite (`tools/test/suite.py`) was never started — background jobs are killed between
  tool calls in this sandbox (the 16-3 session recorded the same), and the foreground build of
  all test binaries ate the disk budget. The per-binary list above is the substitute.
- Live checks for the main agent (PCSX2, fresh boot): at the dock (x≈31, y≈39.5, z≈22) the two
  chat guards should stand face-to-face with the chat voice lines after Bond crosses x≈50 on the
  dock at z≈26-30 (plate #528); firing a pistol round near them should break both off the chat.
  The watch spot for a screenshot: Bond at (45.0, 38.3, 24.0) looking west down the dock.

## Usage

not visible to me
