# smash-stat-gate-02 — the named smashable on 07000002 counts in Bond's accuracy

Task: `Destroy_Smash` 0x12bb60:42-55 calls `PlrStat_LogShotHitScenery(0)` on 0x07000002 when a
switch-channeled smashable is destroyed and its obj+0xd8 equals `hashtable_getitem(0x20001d9)`.
The port had no such arm (the level-gates-world report recorded it open — destroy leaves carried
no object-name hash). This task ports the arm and pins it through the real path.

## What the game does (the evidence)

`Destroy_Smash` 0x12bb60 [M], the destroy path's death block (disasm 0x12bbe0-0x12bc5c, checked
against the C — the decompile's stores are all present):

1. the smashed latch, the light delete, `Effect_RemoveSObjs`, the glist swap
   `Control_SetGList(obj, BL[0])` when BL[0] != 0 [M 0x12bbb8-0x12bbc4],
2. the timer negate, then — for a tag with a channel word — `switch_channels[ch] = 1`
   [M 0x12bb60:40],
3. the stat arm [M 0x12bb60:42-55, disasm 0x12bc2c-0x12bc48]: only on
   `GameState._12_4_ == 0x7000002`, only when `obj+0xD8 != 0`, only when
   `obj+0xD8 == hashtable_getitem(0x020001d9)` — then `PlrStat_LogShotHitScenery(0)` (arg 0
   from 0x12bc48 `move $a0, $zero`; the function is 0x198408 [M symtab; the task text's
   0x1998d8 is `PlrStat_LogShotFired`, the +1 side of the same tally]).

`PlrStat_LogShotHitScenery` 0x198408 [M]: behind the MP/Mission_Status gate, the per-player
tally (`PlrMissionStats+0x04`, the accuracy denominator) decrements once, floored at 0, and the
+0x08 running counter re-clamps down to it. The port's function already existed
(crates/nf-game/src/progress.rs `log_shot_hit_scenery`, citing both game callers) — the missing
piece was this call site.

**What obj+0xD8 is and how it comes to equal the named glist** — the chain the port had to
carry:

- The map parser registers **hashcode → celglist pointer** [M 0x1d1508:28-29
  `hashtable_additem(*(int*)(FileNextBlock+4), pCurrCelList)`], so `hashtable_getitem(0x020001d9)`
  returns a glist pointer.
- The map-data walker computes `pCelList + placement_object_index·0x50 + 0x50` — the **glist
  SIBLING** — and passes it to the placement creator [M 0x1d1b68:63,96]; the dispatcher's
  `move $t0, $t1` [M 0x1d04fc] hands it on as `Destroy_Create`'s 5th arg.
- `Destroy_Create` 0x12c318:25-30 stores the target glist's +0x40 into the tag's +0x00 **unless
  that word is 0, in which case the sibling**. At runtime the +0x40 word of placement-created
  glists reads **0** [L ee_snapshot.bin: the 495-object walk, 4 distinct glists, all +0x40 ==
  0] — so the sibling wins, and the smash's `Control_SetGList(obj, BL[0])` swaps obj+0xD8 to
  the sibling before the compare. (The 5th arg is otherwise an unset register [M the
  dispatcher's 0xe1 arm sets no stack arg — 0x1d0660-0x1d0674]; the doc's artifact note in
  docs/fields/destroy.md stands.)
- [D] On 07000002 the name 0x020001d9 resolves to **"Computer2"** (map 51 object 12) — the
  glist sibling of **"Computer1"** (map 51 object 11), which is destroy placement pl[14]'s own
  target (hash 0x020001d8, param 0 = 72 = 0x48, the channel Mayhew's computer writes). The
  level's other destroy pairs follow the same shape ("kitchen-extinquish1"→"-1b",
  "cable light1"→"-1b" — smashed-model hashes 0xffffffff, the compare misses). 0x020001d9
  appears on no other object of the level [D the map-51 sweep].

## What changed

1. `crates/nf-game/src/props.rs` — one field on the destroy leaf: `smashed_hash: u32` (the
   sibling object's hashcode, computed in `Props::create`'s 0xe1 arm from the target object's
   next parse-order neighbour — no level-wide name table). `Props::smash` sets a one-shot
   scratch (`Props::stat_hit`) AFTER the channel write when the leaf's smashed hash is
   `SMASH_STAT_HASH` 0x020001d9 [M the compare at 0x12bc3c-48]. The game's level gate lives at
   the reader (the sim owns the level id).
2. `crates/nf-game/src/drone.rs` — `Drones::smash_stat_check` consumes the scratch on all four
   smash paths (the bullet kill through `Destroy_CollisionHandler` → 0x12bb60, the
   blast-propagation case 0x21, the imp blast, the external-kill cascade through
   `Destroy_Update`'s watch arm): gate `level_current == 0x07000002` [M 0x12bb60:42], then
   `progress.log_shot_hit_scenery(mission.status())` — the function's own MP/status gate inside
   [M 0x198408:11-19]. The order is the game's: the channel write first, the tally after, both
   inside the same tick.
3. `crates/nf-view/tests/smash_stat_gate_02_live.rs` — the pin (below); plus a props unit test
   (`the_stat_scratch_rises_only_on_the_named_smashed_hash`).

No RNG draw anywhere on the path (a constant compare + a counter decrement); the fixed 60 Hz
untouched.

## How it was verified

- **Red before green** [M the arm's effect]: with the log call disabled
  (`smash_stat_check`'s gate forced false), `the_named_computer_smash_drops_the_accuracy_denominator_once`
  FAILS — the real-path tally read `presses 1, fired 0 → 1 (delta 1)`; with the arm it reads
  `delta 0` and passes. The sister leg passes both ways (the negative control).
- **The real path** (the headless stats convention — the real `nf_view::load` of
  07000002.bin, the real `step_bond` trigger pulses): the intro cut-scene is skipped the game's
  own way (`drones.scripts.skip_request` while `sim.disabled` — the player's skip action
  [M Script_FFwd 0x1dac78's chain; the stream-7 ch-0x47 reveal lands during it
  [D 07800002 0x06000084 stream7]]); the corridor vantage [L this session's sweep: from
  (−54.0, 5.9, −23.1) the settled stance puts ONE press into the leaf — dead + ch 0x48; the
  (−55.1, …) walkway end's muzzle sits in the walkway trim and never spawns] — then the trigger
  until the death channel rises.
  - `the_named_computer_smash_drops_the_accuracy_denominator_once`: ch 0x48 rises, the leaf is
    dead, and the tally arithmetic is the game's — `fired == presses − 1` [M 0x1a4bf0:258 +1
    per bullet spawned; 0x12bb60:55 −1 on the named smash]. The leaf-killing bullet's own
    contact is not a decrement [L the sweep's one-press read: delta 0 = the spawn and the
    smash only].
  - `another_smashables_break_leaves_the_tally_alone`: the bunker cable light (smashed-model
    hash 0xffffffff [D pl[394] → "cable light1b"]) breaks under the same real path and the
    tally keeps every bullet (`fired == presses`) [M 0x12bb60:51-54 — the compare misses].
  - The accuracy row on the results data is `hit × count / fired` [M Plr_Stats 0x1984b0:74-78;
    the port's `get_score` row 5] — the pinned denominator IS the figure's input; with no
    enemy hits the row itself reads 0 exactly as the game's would.
- **Per-binary runs** (each binary whole, `--test-threads=1`, NF_ISO set):
  - `smash_stat_gate_02_live` **2 passed / 0 failed** (new)
  - `walkreal_07000002_live` **3 / 0 / 1 ignored** — the full castle walk with the arm in
  - `nf-game` (lib + every integration binary) **1098 passed / 0 failed**
  - `interactables_07000002_live` **18 / 0**
  - `destroy_channels_02_live` **1 / 0**; `destroy_wreck_07000014_live` **3 / 0**;
    `destroy_draw_keys_0700000d_live` **3 / 0**; `fix_blast_scripts_live` **3 / 0**;
    `0700000d_gunimp_door_live` **5 / 0 / 1 ignored**
  - the stats set: `fix_progress_live` **4 / 0**; `progress_remainder_live` **5 / 0**;
    `mission_stats_carry_live` **2 / 0**; `fix_scores_handoff_live` **1 / 0**;
    `fix_progress_detected_hook_live` **1 / 0**; props lib tests (incl. the new identity-gate
    test) inside the nf-game 1098.
- **The full suite** (`tools/test/suite.py`, this tree — master 22d5836b7 + this task's diff;
  no merge needed, master has not moved): see below.

## The full suite

(placeholder)

## What's left

Nothing on the task statement. Notes for the record: the port's shipped world-hit fold
(drone.rs's per-bullet `log_shot_hit_scenery`, fix-progress) is a port model the game's own
census does not back (the game's only `LogShotHitScenery` callers are SP_Update 0x1dbda8:261 —
scripted targets 0x60007e4-0x60007ea taking hit damage — and `Destroy_Smash` 0x12bb60:55);
this task does not touch that fold, and the pin above holds within it. [S unverified, out of
pass/fail: whether the game's wall shots really count into the accuracy denominator live.

## Live checks for the main agent

1. **The stat drop, fresh boot** [M 0x12bb60:42-55]: on 07000002, shoot Mayhew's computer
   (the bunker corridor, ch 0x48) and read `PlrMissionStats[0]+0x04` (0x2bcc24) before/after:
   the destroying bullet's tick drops the word by one MORE than a wall shot's does. Then smash
   the bunker cable light (pl[394]): no extra drop. On any other level (e.g. 07000005's shot-out
   scenery), a smash never drops it.
2. **The smashed glist identity**: gdbwatch write on obj+0xD8 of the computer's Destroy object
   during the smash — expect the swap to the "Computer2" glist (the sibling) at 0x12bbb8-bc4,
   and the 0x12bc3c compare passing against `hashtable_getitem(0x020001d9)`.
3. No render surface: the arm writes a counter only. (The smash's model swap itself was already
   the destroy family's render surface.)

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
