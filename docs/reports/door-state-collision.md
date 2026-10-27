# door-state-collision — the filter landed and verified on its own gates; one probe regression found and bracketed (A/B against master), mp digest does not move

## What's done

1. **The door-state filter landed** (from `origin/s/capsule-cull-and-los-masks-5`
   commit 1475fc25 onto master dcc4a733; this branch is exactly master + this fix +
   docs): `drone.rs` `set_dynamic`'s doors extend now filters by
   `x.is_closed(&drones.doors.group_states)` — the game's `Door_IsClosed` 0x1367b0
   rule [M door.rs `Door::is_closed`; the zone consumer 0x135738/0x1336f8]: a kind-1
   door always reads closed, a grouped door reads `DoorGroupsStates[g] == 2`, the rest
   read `obj+0xF4 == CLOSED` exactly; OPEN/OPENING/CLOSING leaves do not block.
   None of that branch's cull/LOS/gt.rs changes came across (verified: the other five
   drone.rs hunks, gt.rs, world.rs, walker.rs and all live-test diffs were left out).
2. **The SS parts review landed as a comment** at the SS extend (drone.rs): the safe
   door (#205, ch 136) and the doorway bolt (#217, ch 137) stay UNCONDITIONAL — the
   game's script-entity collision path has no state gate:
   `Script_CreateEntity` 0x1d8968's fallthrough
   `hashtable_set_object_to_entity_gfx` 0x1ca6d8 lands the glist whatever the script's
   playhead state is, and every moved entity is rebuilt + relinked per tick
   [M 0x1d8968 / 0x1ca6d8 / 0x1ca798 / 0x1d9a08]. The swept open pose is the clearance
   mechanism (the walk passes "before the safe" → briefcase only after ch 136 opens
   #205) — walkreal_07000008_live's briefcase leg is the empirical check, green.
3. **The task's gates, all run on this tree:**
   - `walkreal_07000008_live` **4/4 GREEN** (the briefcase walk included — the stall
     the filter fixes; 138 s).
   - The **door family** (14 nf-view binaries): 33 passed / 0 failed / 1 pre-existing
     ignored (npc_doors escort-reach, documented in the test) — fix_npc_doors 1,
     npc_doors 6, portal_links 3, portal 2, react_path 1, aim_ray 2, text_labels 4,
     hint_boot 2, glist0 2, remainder 4, cutscene_14 2, party_lock_07 2, rook_06 1,
     rook_06_walk 1. Plus the 6 nf-game door tests: 11 passed / 0 failed / 1 ignored.
   - **Every other walkreal** (all 23 walkreal_*_live binaries 0001…004a + satchel):
     **66 passed / 0 failed / 5 pre-existing ignored**.
   - **mp**: `mp_net_slots_post_move_live::four_human_session_digest_is_pinned` runs
     **EXACT**: tick0 0x5c0bb60a8cdf8938, final **0xfb7b58f73e340ec3 — master's
     digest did not move** (the filter is inert on Ravine 0700004b: no doors, as
     capsule-cull-5 measured). The sync-sensitive mp family spot-run green:
     mp_core 3, mp_world_sync 5, mp_bots 8, mp_bots_moving 3, mp_bodies 3,
     mp_16_slots 3, mp_shot_gate 5, net_e2e_loopback 2, net_resync 3 — 35/0.
   - `nf-game --lib`: 578 passed / 0 failed / 5 ignored.
4. **The full suite (chunk_suite.py — background runners die at the tool-call
   boundary in this sandbox, as capsule-cull-5 reported): 303/844 binaries recorded,
   1081 passed / 1 failed / 84 ignored** when the window closed. The one non-OOM
   failure is item 5 below (a real, NEW finding). The 3 slim-ISO `disc_golden`
   failures are the allowed ones.

## The new finding: probe_07_route regresses UNDER the filter (A/B proven)

- `probe_07000007` route probe: **GREEN on master (revert-tested in this sandbox),
  RED with the filter.** The walker gets stuck at "the hall's north-west corner":
  bond=(94.17, 44.14, −47.02) hovering (grounded=false) over `floor_below 43.00006`,
  target=(92.60, 43.60, −47.50) d=1.64, 6000-tick cap. The nearby door:
  `door@(94.9, 43.0, −47.2) kind=0 state=0x4(OPEN) pos01=1.00 ch_open=0 ch_lock=0
  lock_flag=0 approach=0 lock_on=false`.
- Reading: that OPEN kind-0 leaf at pos01=1.00 sits where the hall floor (44.13,
  from the route's own walk logs) meets a 1.1 step down through the doorway (the
  lower floor 43.0). With master's always-solid leaf Bond crosses (the leaf's open
  pose is load-bearing there — a bridge/sill); with the filter the leaf leaves the
  collision set and Bond hovers at the step edge until the cap. **So the two gates
  pull the filter in opposite directions**: walkreal_07000008 needs open leaves
  non-solid (door #10's open leaf spans the doorway), probe_07_route needs THIS open
  leaf solid.
- Mechanism conclusion (for the next round, [S] until pinned): `Door_IsClosed`
  0x1367b0 is the game's **zone** rule [M 0x135738/0x1336f8 — that is where the
  decompile shows it], but the movement-collision set likely keeps the leaf ALWAYS
  solid (`Collide_PickObj` walks the cel-linked objects "whatever moves them" [M,
  the fix's own comment]) with passability coming from the **pose** — exactly what
  this task's SS review concluded for the script entities ("the swept open pose IS
  the clearance mechanism" [M 0x1d8968/0x1ca6d8]). Under that model the 07000008
  stall's root cause is door #10's OPEN POSE in the port (its open matrix still
  spans the doorway — [D] the keys: the 5×6 panel at x 155.003, z −118..−123), not
  its solidity gate. The state filter is the wrong layer for that fix, but it IS
  what unblocks walkreal_07000008 today; this report pins both facts so the next
  window can replace the filter with the pose fix consciously.

## What's left

1. **Reconcile the gate**: pin the game's movement-collision rule for open leaves —
   live-check door #10 (07000008) and the 07000007 hall door's `obj+0x90` matrix vs
   their port matrices while OPEN (see "Live checks"), then either fix door #10's
   open pose and drop the filter, or gate the filter by door kind/group semantics
   with evidence.
2. `mp_online_failures` OOMs in this sandbox (see Verification): needs a >3 GB box.
3. The suite tail: 541 binaries not reached in this window (see the tally above);
   the touched families were all run explicitly and are green apart from item 5.

## How it was verified

- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000008_live --
  --test-threads=1` → 4/4. The 14 nf-view door-family binaries → 33/0/1ign. The 6
  nf-game door tests → 11/0/1ign. All 23 walkreal_*_live binaries → 66/0/5ign.
  The mp pin → exact (tick0 and final both match the constants). The mp sync spot
  set → 35/0. `nf-game --lib` → 578/0/5.
- A/B for the regression: `git revert --no-commit 1dd5ac04` (master behaviour)
  → probe_07_route GREEN in 3.6 s; `git reset --hard HEAD` (the filter back)
  → the STUCK above. Both runs in this sandbox, same target dir.
- Full suite: `tools/test/chunk_suite.py` (state
  `/home/z/my-project/scratch/door_state_suite_state.json` — scratch, not
  committed) looped in the foreground: **303/844 binaries, 1081 passed / 1 failed /
  84 ignored**. FAILED lines: `probe_07_route` (the regression, above) and
  `mp_online_failures` — **OOM-killed by the sandbox's 3 GB cap** (dmesg:
  `oom-kill ... task=mp_online_failu ... anon-rss:3498688kB`; it loads arena
  0x07000023 on several machines — a memory-ceiling kill, not an assertion; the
  filter only REMOVES collision parts, so it cannot add memory). No TIMEOUT lines.

## Live checks for the main agent

1. **07000008 Secure Room door #10, while OPEN (after ch 137)**: watch
   `obj+0x90` (the door's collided pose matrix) and the leaf's world corners — does
   the game's open leaf span the doorway plane (x≈155.0, z −118..−123) or swing
   clear? If it swings clear in the game while the port's matrix stays in the
   plane, the port's open-pose math is the 07000008 root cause, not solidity.
2. **07000007 hall door (the probe_07_route one, at pos01=1.00/OPEN)**: same dump —
   expect the open leaf to lie where Bond walks (its top face ≈ y 44.1 over the
   1.1 step at (94.2..94.9, −47.0..−47.5)); confirm the game's leaf stays in the
   collide set while OPEN (a Bond walk across it with collision-watch on
   `Collide_PickObj` hits on that object).
3. Render check for the walkreal_07000008 merge: `--bond 154.0,43.1,-120.7,0,0`
   looking east after ch 137 — Bond should stand in the open doorway, not pressed
   against a leaf panel.

## Usage

not visible to me — this session's driver surface shows no token, cost or
context-window figures (the IM gateway metadata carries only session/chat/trace
ids).
