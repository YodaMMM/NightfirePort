# level-03-audit — the 07000003 behaviour audit

Branch `s/level-03-audit` (off origin/master b312d959 — master did not move during the session, so
the final tree IS merged with master). Sandbox 22:29–00:28 UTC 2026-10-25.

## What's done

1. **The full type census of 07000003's main map** (map 50, **511 placements**, `nf-extract
   placements-all` [D]) against `parsemap_create_dynamic_objects` 0x1d04f0 [M], one row per kind with
   the creator, the placements, the game's behaviour and the port's module — written to
   `docs/levels/07000003-audit.md`. 31 kinds present: 20 characters 0x0F (enables p3: 0x25×4, 0x21×7,
   0x23×3, 0x24×2, 0x4E×2, 0×2; deaths p12: 0x60×3 protected, 0x84, 0x86, 0x3A), 97 breakables,
   13 doors, 24 cover nodes, 19 TouchOnce plates, 8 SS, 5 destroyables (+ their 5 scriptless
   0x31 damaged twins — never created, same rule as 02 [M 0x1b0448]), 5 pickups, 3 SPs, 3 rigid
   bodies, 1 ladder/exit/dynobj/rain-generator/AI-volume, the AND/SEQ muxes, sound/music triggers,
   hints, rainboxes, sky — and the zero-kinds cross-checked against the full dispatcher. Maps 0-49
   are the prototype/prefab maps (Switch/Lock/camera templates at the origin), not level content.
2. **The scripts decoded** (`nf-extract nis 07000003.bin` — the level's 9 NIS entries live INSIDE the
   level archive; no 078/079 companion packs exist for 03 [D files-manifest]):
   - **SP #475 → 0x06000080** (mode 0, mask 4, **out ch 0x36**, stay): the keyed-camera cut-scene
     (Bond anim 0x06000552 skin 0x05000009 + guard 0x06000551 skin 0x05000039, 244 camera keys,
     Events 4/5/6/7/0xb[0x1e]). Trigger = `SP_Update`'s OUT-channel read [M 0x1dbda8:70-74]; the
     writers are plates #416/#417 (out 0x36, locked while guard 394's death ch 0x3A is on [D p7]).
   - **SP #109 + #110 → 0x06000563** (the Dragon Safe reveal; #110 is mode **64 = preload-then-prime**
     [M 0x1db9a8:76-85 → primed state 1], mask 4 = the `SP_Activate` E-use latch [M 0x1dc338:82]):
     animates the dragon entity 0x020005f0 + wall 0x020005db, **spawns the safe-key pickup**
     (Entity obj_type 0xf0, params [2,0,0,0x4e,0xffff,0] → kind 2 key, +0x28 = 0x4E [M Pickup_Create
     0x18dda0:121 + Pickup_Handler 0x18e650:330-333]), writes ch 0x96 at t77, sound 473.
   - The other 6 entries (0600006f alarm-switch swap, 060007c4/06000027/06000052 keyed choreographies,
     0600005e camera, 06000564 loop, 0600004f) are unplaced anim/sub-scripts; every op kind they use
     is ported [M nis.rs dispatch]. No gap.
3. **The mission record** cross-checked (`mission::c07000003::DEF` [L ee_start_05.bin @ 0x2a3f18]):
   obj0 0x4E / obj1 0x51 / obj2 0x53, reveals 0x4D/0x4D/0x52, no per-objective fail. The channel
   graph now closed on the disc: **0x4E = the spawned safe key's take channel; 0x52 = AND #435
   (0x4E AND 0x51); 0x51 = AND #436 (0x4F AND 0x50 — the un-ported mission/dialogue layer)**; the
   exit #451's key gate is 0x52 [D]; the SEQ pair #472/#474 (touch 0x83/0x85 THEN death 0x84/0x86 →
   0x68/0x69) feeds SoundTriggers #490/#491 (snd 333) [D].
4. **GAP-2 found and fixed**: `07000003.md` claimed "the writers of 0x4E/0x51/0x52 are not on the
   disc". Wrong for 0x4E (the script-spawned key writes it — a script-created object a placement-only
   scan misses) and for 0x52 (written by trigger AND #435; the old fix-finale-blockers enumeration
   bounded `switch_channels` code writers, not trigger outs). Doc corrected with the citations.
5. **GAP-3 fixed**: the level's chains were unpinned. New `crates/nf-view/tests/wiring_07000003_live.rs`
   (4 tests, green) pins through the real level path: the AND/SEQ outs+inputs, plate #471's lock 0x90,
   plate #504's mask-4/0x31, door #4's lock channel 49 vs #5's 0, SS #502's mask-1/ch-49/state-0x40
   semantics, the mission record rows, and the spawned key's params (kind 2, +0x28 0x4E) from the
   loaded script library.
6. **GAP-1 documented (large, pre-dispositioned)**: 3 × 0xD8 `secondgarden_bottom_lamp` rigid bodies —
   `rigid_body.rs` holds the init math [M 0x1d7ad8/0x1d7ba0/0x1d7da0] but nothing integrates it. Same
   open item 01/02 shipped with; its own task.
7. **Stale comment corrected**: `walk_07000003_live.rs`'s header claimed the ch-0x52 writers were not
   on the disc — updated to the AND #435 finding; the test itself was already right and stays green.

## How it was verified

- `wiring_07000003_live` 4 passed; `walk_07000003_live` 2 passed; **`walkreal_07000003_live` 4 passed
  in 334 s** — the full real-input walk (spawn → the locked secret-bedroom gate via plate #504's ch
  0x31 → SS #502's picture → servants → the exit E with ch 0x52), which independently exercises the
  audited chain end-to-end; `music_triggers_live`, `mission_unlock_flow`, `simplescript_live`,
  `nis_event9_live`, `nis_page_boot_live`, `pickups08_live`, `trigger_mask2_census_live` — all green.
- Every assertion in the audit cites [D] (placements-all / nis decode / files-manifest) or [M]
  (function + address); the port-side statements cite the shipped modules. No [S] item entered
  pass/fail. One refinement during the work: SS #502's param row was first mis-read as E-usable;
  the param map [M SS_Create 0x1a9c58: mask = p2, bit 0 = don't-write, bit 2 = E-use] settles it as
  ch-driven — the audit doc carries the corrected reading.

## The suite

The sandbox kills background jobs (three nohup `suite.py` starts died before the first line), so the
suite ran in foreground chunks (`/home/z/my-project/scripts/chunkrun.py`, the runner's semantics:
each binary `--test-threads=1`, NF_SILENT=1, 560 s cap, resuming from a done-file over the chunks):
**279 of the 486 built test binaries ran — 911 tests passed, 2 failed, 22 ignored** (chunks of 114 +
89 + 50 + 16 binaries + the 10 targeted level binaries; 618 binaries exist on disk, 486 unique test
names). The two failures are unrelated to this task's diff (which contains NO source change — docs +
test files only, so both are pre-existing on master):
- `fix_0c_pylon_live` (2 tests, rc=101) — 0700000c pylon views, untouched by this branch;
- `mp_stress_explosives_live` (rc=-9, SIGKILL at the 560 s cap) — the known slow binary (the 02
  session re-ran it green with more time).
The remaining **207 binaries did not get a slot** — they are the slow `_live` walks (the completed
chunks consumed the budget; the level's own three walk/walkreal/wiring binaries all ran green above).
The next session can resume with the same runner or `tools/test/suite.py` directly.

## What's left

- The 207 remaining suite binaries (no failures expected; no source diff against master).
- GAP-1 (rigid bodies) as its own task.
- The un-ported mission/dialogue layer that owns ch 0x4F/0x50 (obj1's AND inputs) — pre-existing,
  noted in the audit doc.

## Live checks for the main agent

Nothing is blocked on a live check. If a render pass happens anyway, useful spots (`--bond
x,y,z,yaw,pitch`, level 07000003):
- **(6.7, 6.1, -42.5) facing the dragon** (`masterarea_dragon`, SP #110): the reveal should show the
  bookcase/dragon swung open (script 0x06000563's keyed path) with the safe-key pickup on the plinth.
- **(11.8, 6.0, -35.6) facing the picture wall** (SS #502 `bedroom_triggerpicture`): after plate #504,
  the picture swung aside and door #4 `door_secret_bedroom` (14.68, 6.65, -35.40) unlocked.
- **(18.1, 6.6, 16.0)**: the exit Load Point #451 — E refused without ch 0x52, loads 07000004 with it.

## Files

- `docs/levels/07000003-audit.md` (new — the audit: census, scripts, mission record, GAPs)
- `docs/levels/07000003.md` (GAP-2 correction)
- `crates/nf-view/tests/wiring_07000003_live.rs` (new, 4 tests green)
- `crates/nf-view/tests/walk_07000003_live.rs` (stale comment corrected)
- `docs/WORKLOG.md` (the entry)

## Usage

not visible to me — this session's driver interface shows no token, context-window or cost figures.
