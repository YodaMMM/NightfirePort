# pickup-medal-gate — the report

Branch `s/pickup-medal-gate` (from origin/master 82362142, merged origin/master 268c12a6 at
5cb31d34). Task: kind 6 — the '007' bonus pickups — is gated by the level's gold medal in the
game; the port ignored the gate. This task ports the gate and the medal state it reads.

## What the game does (the evidence)

- `Pickup_Create` 0x18dda0:88-104 [M]: after the glist check and the id −1 check, and BEFORE the
  7..=11 set intercept: `if ((uVar7 == 6) && (PlrStats_HasGoldMedal__Fv(), lVar3 == 0)) return 0;`
  — a kind-6 create with no gold medal produces nothing.
- `PlrStats_HasGoldMedal` 0x199a40 [M] = `Menu_HasMedal(Mission_BaseMapHCode(), 3, 0)`.
  `Menu_HasMedal` 0x202580 [M] zeroes a 16-byte REWARDINFO and calls
  `PlrStarts_ProcessRewardCounter(&bonus[controller], map, 3, 0, &info)`, returning the `has`
  byte at info+0xc. `PlrStarts_ProcessRewardCounter` 0x199210 [M] walks the 12-entry
  `RewardsTable` (0x2bc920) for the map hash, reads the counter-3 descriptor (u16 bit at +0,
  u16 kind at +2), and tests bit (bit + group rank) of the caller's u64 — the game's `bonus[]`
  global (0x2e1b88, u64[21]; SP reads slot 0). The guard is `if (param_3 < 5)` [M 0x199210:24].
- The BaseMap is NOT the archive id. `Mission_Init` 0x182270 [M] resolves the level to the
  mission layer's `BaseMap` (0x30cba0) over the game's 24-record mission table. The decompiler
  renders that table walk with an off-by-one (it pairs each map with the NEXT record's mission
  id); the disassembly pairs the mission id at 0x2a4350+k·0x28 with the map hcode at +4 and the
  order at +8. Dumped from the snapshot [L ee_start_05.bin 0x2a4350]: **0700000a → 0x07000009**
  (row 8), **07000011 → 0x07000011** (row 15) — both kind-6 levels play under BaseMaps that sit
  in the RewardsTable, so the gate is live for both. `Mission_BaseMapHCode` 0x1830a0 returns
  `BaseMap` while `MPSettings+0x384` is 0 (SP), GameState+0xc in a match.
- The gold bits are kind-4 groups: rows 0x0700005/0x0700009/0x0700011 share descriptor word
  (bit 0, kind 4) [L 0x2bc920 rows 1/5/7] → consecutive bits 0/1/2 by scan rank [M 0x199210's
  packed-word scan]; row 0x0700001/0x070001b share (9, kind 4) → bits 9/10 [L row 4/11].
  Earning the gold sets the level's own bit only (`Menu_SetLevelBonus` 0x1ffa10 [M], counters
  1..=medal, set-if-clear).
- The save keeps the word: `LS_LoadBonus` 0x1ce488 [M] pulls 2×32 bits from the BUNS block into
  one u64 and `Menu_SetBonus(bits, slot, 0)` replaces `bonus[slot]`; nothing resets `bonus[]`
  per map (the writers are the save load, the PTP boot restore and the award path) — one store
  that survives every level. Fresh boot = the BSS zeros [L ee_start_05 0x2e1b88 all zero].
- The take side needs no separate gate: `Pickup_Handler` 0x18e650's case 6 [M :234-236] runs
  only for pickups that exist, and `HasGoldMedal`'s only caller is `Pickup_Create` — no medal,
  no pickup, nothing to take.

## What the port does now

- `nf-game/src/pickup.rs`: `Pickup::create_placed` gained `has_gold_medal: bool` and refuses
  kind 6 without it, in the game's check order [M 0x18dda0:88-104]. Kinds 7-11 and every other
  kind are unaffected; the id −1 check still runs first.
- `nf-game/src/progress.rs`: new `medal_set(bits, hash, kind)` (the `Menu_HasMedal` lookup over
  explicit bits — a level with no RewardsTable row or kind ≥ 5 reads nothing, exactly the
  game's zeroed-info arm [M 0x199210:24]) and `has_gold_medal(bits, hash)` (the
  `PlrStats_HasGoldMedal` shape). `Progress::has_medal` delegates to it. Fixed on the way: the
  old `counter > 5` / `kind > 5` guards let counter/kind 5 index `desc[5]` out of bounds; the
  game's guard is `param_3 < 5` [M 0x199210:24], so the port now rejects ≥ 5.
- `nf-view/src/scene.rs`: `DroneContext.bonus: u64` (the `bonus[0]` word in play) +
  `DroneContext::has_gold_medal(level_id)` — the `crate::mission::LEVELS` row's `mission` hash
  (the port's copy of the mission table [L ee_start_05 0x2a4350]) then the gold bit. Both
  pickup-build paths (the placements and the script reservations) thread it into
  `create_placed`. A level with no mission row (the front end, MP maps) plays BaseMap −1 → no
  RewardsTable row → the gate stays shut, as the game's [M 0x182270's state-5 arm].
- `nf-view/src/lib.rs`: `load_impl` carries the word; the front end's launch passes the active
  profile's `bonus[0]` on EVERY load (`load_with_carry_medals` + `FrontEnd::bonus_bits`) — the
  game's one-store behaviour [M LS_LoadBonus / Menu_SetBonus's writer set]. The plain `load*`
  wrappers keep 0 = the fresh-boot BSS state [L], which is what replay parity pins.
- The profile side needed no change: `sync_profile_progress` already clones the sim's whole
  Progress (the award path `set_level_bonus` writes the bits) and the Dossier/Rewards pages
  already read the profile's bits through `has_medal`.

## How it was verified

- New `crates/nf-game/tests/gold_medal_gate.rs`, 3 tests, all green on the final merged tree:
  the medal math headless (no bits → no medal for every row; the award arm lights the level's
  own bit and not the sister levels' — the 0x0700005/9/11 group's ranks; bronze-only is not
  gold; the mission-table BaseMaps for 0700000a/07000011 gate shut then open; the no-row and
  kind-5 arms read nothing) and the create gate over the disc's weapon_data (kind 6 + no medal
  → None [M 0x18dda0:93-95]; medal → Some with the kind-6 spin [M 0x18dda0:138-144]; kind 0
  unaffected; id 0xFFFF still refuses first [M 0x18dda0:89-91]).
- Eight existing live tests pinned the port's UNGATED create — the decompile contradicts them,
  so per the rules the TESTS were fixed, each with the address: the kind-6 walk-over tests on
  07000011 (#275), 0700000b (#300), 0700000d (#439), 07000007 (#502/#503), 07000005 (the census
  + the valley token), 07000001 (the 007 take + the model census + the kind census) and
  fix_progress's tally test now load through `load_with_carry_medals` with the level's gold
  word (built by the port's own `set_level_bonus` — no magic bits), and each gained the
  no-medal sister: the same load with the fresh-boot zero word creates NO kind-6 [M
  0x18dda0:93-95]. Re-ran green on the final tree: `interactables_0700000d_live` 11/11,
  `gold_medal_gate` 3/3, `pickup_live` 1/1 (nf-game, nf-view).
- `load_scene_equality` re-pinned per its own protocol (the pin is marked "[S port
  engineering] … no game claim"): every drifted level lost exactly its kind-6 rows' meshes —
  −418 verts / −314 indices / −1 material per 007 token (07000002 −3, 07000005 −2, 0700000c −4,
  07000001 −2) and 07000014 −2 tokens (−836 verts, pickups 32 → 30). Re-ran green on the final
  tree.
- The crates touched: `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
  -p nf-view -- --test-threads=1` is the suite's command; the suite (`tools/test/suite.py`,
  NF_ISO set) was started at 16:37 UTC and runs against this branch (see the suite section).

## The suite on this tree (partial — the sandbox deadline cut it)

`build ok: 829 test binaries in 1050 s`, then the per-binary run. At 17:55 UTC (the report's
final read) it had reached the `fix_*`/`s*` range with:

- FAILED `disc_golden::bigf_header_fields_behave_as_documented`,
  `disc_golden::bigf_tables_match_the_golden_manifest`,
  `disc_golden::directory_tree_matches_the_golden_manifest` — the 3 slim-ISO failures the task
  allows.
- FAILED `identity_disc::the_real_disc_passes_the_identity_check` — environmental, pre-existing
  on this sandbox's image: the test pins the FULL disc's 1,191,072 sectors
  [D docs/build-identity.md], the sandbox has the on-foot slim slice (1,061,754,880 bytes =
  518,435 sectors). Not from this task; the test is [D]-correct for the full disc.
- CRASH `mp_online_failures (exit -9)`, CRASH `net_soak_faults_live (exit -9)` — the kernel
  OOM killer (SIGKILL), not a Rust panic, in the net-session binaries. This sandbox has 4 GB
  and the net tests spawn full session sets; suspected environmental (the crash mode and the
  binaries are unrelated to the pickup gate — the gate only changes how many kind-6 pickups a
  level load creates). Flagged for the driver: worth one clean-suite look on a bigger sandbox.
- FAILED `fix_0c_pylon_live` (both pixel tests) — environmental: this sandbox has NO vulkan
  ICD (`/usr/share/vulkan/icd.d/` is empty; wgpu `request_device` →
  `NotFound { active_backends: Backends(0x0) }` — the panic is at device creation, before any
  scene work). Every GPU-rendering test the suite reaches will fail the same way here (render*,
  view_*); the lessons list says visual work needs the main agent's GPU render anyway.
- FAILED `load_scene_equality` / `interactables_0700000d` in the suite run — both were the
  pre-fix binaries (the suite built at 16:55, before the re-pins); both re-ran green on the
  final merged tree as noted above. Same for `fix_pickups_remainder_live`'s all-level census:
  it expected every 0xF0 placement to create — the kind-6 rows included — which the decompile
  contradicts [M 0x18dda0:93-95]; the expected set now skips the gated rows (fresh-boot word)
  and the test re-ran green on the final tree (35.7 s). Likewise
  `interact_0700000c_live::the_static_world_kinds_all_register` pinned "pickups 15 [D]" — the
  disc's PLACEMENT count; the fresh-boot load creates 11 (the four kind-6 rows gated [M
  0x18dda0:93-95]) — re-pinned and re-run green.

No total line yet — the run is longer than the window. The next message can resume from the
branch and re-run `python3 tools/test/suite.py` for the full pass (the tree is merged with
origin/master 268c12a6).

## What's left

- The suite's total line on the final tree (partial results above; the fixed binaries are
  verified green standalone).
- One [S] noted in code: a Play-without-saving session's awards live only in the sim's own
  Progress in the port; the game's in-memory `bonus[]` globals would carry them into the next
  launch (the same note `unlock_mp_settings` carries). Modelling that needs the front end to
  hold a session Progress — out of this task's scope.
- Render check (lesson 6): the gate's visible effect — with a save holding 0x07000011's gold
  medal, 07000011's two 007 tokens (#275/#276) appear and spin; with a fresh save they do not.

## Live checks for the main agent

- On a fresh boot of 0700000a (no save): `bonus[0]` @ 0x2e1b88 should read 0
  [L ee_start_0700000a.bin], and the three "Pickup 007" placements at (9.41, 0.29, 31.97),
  (34.75, 0.29, 34.38), (−38.81, 0.29, −3.31) [D docs/levels/0700000a.md] should NOT create —
  the pickup list (Bond's take never fires on them).
- After loading a save that holds 0x07000009's gold medal: `bonus[0]` bit 1 set (the
  0x0700005/9/11 gold group's rank 1 [L 0x2bc920 row 5; M 0x199210]) and the three tokens
  present on 0700000a.
- gdbwatch on `Pickup_Create` 0x18dda0 with a kind-6 placement loading: the
  `PlrStats_HasGoldMedal` call at :93 returns 0 on the fresh boot and 1 with the medal —
  a read watch on BaseMap 0x30cba0 at level start should show `Mission_Init` 0x182270 writing
  0x07000009 for mission 0700000a (the disasm pairing this task pinned).

## Usage

not visible to me — this session's environment shows no token, credit or context-window
figures anywhere (no usage banner, no cost line in the transcript or the tool output).
