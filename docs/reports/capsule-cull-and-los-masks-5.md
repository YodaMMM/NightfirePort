# capsule-cull-and-los-masks-5 — the door-state fix LANDED (walkreal_07000008 4/4 green), the mp digest divergence bracketed to the 4a155ddf merge content

## What's done

1. **Round 4's fix recipe landed** (crates/nf-game/src/drone.rs, `set_dynamic`'s
   `script_parts`): the doors extend now filters by
   `x.is_closed(&drones.doors.group_states)` — the game's `Door_IsClosed` 0x1367b0 rule
   [M door.rs `Door::is_closed`; the zone consumer 0x135738/0x1336f8]: a kind-1 door
   always reads closed, a grouped door reads `DoorGroupsStates[g] == 2`, the rest read
   `obj+0xF4 == CLOSED` exactly — OPEN/OPENING/CLOSING leaves do not block. The port
   previously posed every glist door's mesh into the collision set unconditionally, so
   the OPEN Secure Room door #10's leaf (its natural pose = the 5x6 panel at x 155.003,
   z −118..−123 [D the door's keys]) blocked walkreal_07000008's fusebox→safe walk.
   **walkreal_07000008_live is 4/4 GREEN** — `walk_the_gondola_to_the_briefcase_and_back`
   passes in 29.7 s (the stall was 1600+ ticks at (154.50, 43.10..43.42, −120.64); Bond
   now walks through the open doorway, reaches "before the safe", burns both bolts
   within the laser's 4.0 reach, takes the briefcase, and the objective completes).
2. **The SS parts reviewed the same way** (the safe door #205, the bolt #217): the
   game's script-entity collision path has NO state gate — `Script_CreateEntity`
   0x1d8968's fallthrough `hashtable_set_object_to_entity_gfx` 0x1ca6d8 lands the glist
   whatever the script's playhead state is, and every moved entity is rebuilt + relinked
   per tick [M 0x1d8968/0x1ca6d8/0x1ca798/0x1d9a08] — so the SS parts stay unconditional
   (drone.rs unchanged for them). The swept open pose IS the game's clearance mechanism:
   the green walk passes "before the safe" → briefcase AFTER ch 136 opens the safe door,
   which is itself the empirical check that the SS safe door's open pose clears the
   route in the port exactly as in the game.
3. **The door family ran green** — 37 passed, 0 failed, 1 pre-existing ignored:
   fix_npc_doors_live 1/1, npc_doors_live 6+1ign (the escort-reach ignore, documented in
   the test), door_portal_links_live 3/3, door_portal_live 2/2, fix_door_react_path_live
   1/1, door_aim_ray_live 2/2, door_text_labels_live 4/4, door_hint_boot_count_live 2/2,
   door_glist0_e_dead_live 2/2, doors_remainder_live 4/4, cutscene_door_07000014_live
   2/2, party_door_lock_07000007_live 2/2, rook_door_06_live 1/1, rook_door_06_walk_live
   1/1, walkreal_07000008_live 4/4.
4. **The mp digest** (mp_net_slots_post_move_live `four_human_session_digest_is_pinned`,
   the round-4 method run end to end):
   - **tick0 MATCHES the pin** (0x5c0bb60a8cdf8938); the **final moved**:
     0xfb7b58f73e340ec3 → 0xbc5e2155a3786e9c on this tree.
   - The baseline tree at a7afd219 was built (own `CARGO_TARGET_DIR`, worktree
     /home/z/nf-baseline) and its `NF_NET_HASHDBG=0-239` run re-verifies BOTH pinned
     values exactly — the pin is intact at its commit.
   - **The first divergence: session t142 (sim frame 143), component = bodies** — slot 3
     only: x +0.00122, y −0.00235, z/yaw/pitch/substate/crouched identical; the match
     component follows it through `write_net_slot_positions` (the slot record takes the
     post-move body position). **frame_rng is still aligned at the divergence** — round
     3's shared-RNG-draw suspect (guard sight/aim timings) is NOT the first mover. The
     world component never moves across all 240 ticks on either tree (Ravine 0700004b
     has no doors, no takes, no channel writes) — which also proves the door-state fix
     is inert on this level.
   - **Probe exonerations** (env-gated single builds on the fixed tree, run through the
     same diag): the band-settle-3 radial-dir push arm (`NF_NO_RADIAL=1`) — divergence
     persists; the bit-0x20 static collision skip (`NF_KEEP_BIT20=1`) — persists.
   - **Merge-point bisect along master** (the diag harness only compiles on trees
     containing a7afd219 — `net_tick`/`net_rng` postdate the level-14 lineage):
     62aa121a (batch-1024h) CLEAN, 15c0e34a (batch-1026c) CLEAN, 82362142
     (batch-1026d, incl. rigid-body-remainder) CLEAN, 18ca71c8 (batch-1026f) CLEAN —
     each matches the pinned final to t239 — and 4a155ddf DIVERGENT. **The cause is
     inside 4a155ddf's master-side content (the batch-1026g/h/i/j merges: band-settle-2/
     3, guard barks, medal gate, display-hatch-2, rope casings, ride-cutscene-base) or
     the merge's own resolution** — the window's only other commit (1032200a) is
     wording-only. Within that content, exonerated by observation: the medal gate (no
     takes happen; the world fold never moves), the barks (no guards in Ravine MP), the
     debris arm (no explosions), the rope casings (no shots fired), band-settle-2's
     below-B kill (env-gated off by default), band-settle-3's radial arm (probe above).
     **Remaining suspects: 353b1008 (script anim actors riding their SP's matrix —
     script-actor collision parts would move; Ravine loads 109 script objects), the
     display-hatch-2 toucher's final shape, and the 4a155ddf merge resolution itself**
     (both parents heavily edited the collision files in this window).
   - **NOT re-pinned**: the cause is not yet named, so the pin stays red with the
     divergence fully characterized. The next window's sharpest move: build 4a155ddf^1
     and 4a155ddf^2 (both contain a7afd219, so the harness runs) — whichever side is
     already divergent names the branch; if only the merge itself diverges, diff the
     three trees' collision files directly.
5. **Diagnostic landed**: crates/nf-view/tests/mp_digest_diag.rs — the pin's exact
   session shape (same ARENA/SEED/TICKS/script/harness, PORT_4 28361 so it can run beside
   the pin binary) printing the per-tick fold h0, asserting only the two simmed machines'
   per-tick agreement. Runs on any tree containing a7afd219; it is the bisect tool for
   the cause hunt (the component-level variant that names the component is three extra
   lines — `net_component_hashes` — documented in the WORKLOG entry).
6. **Suite**: the background fast runner dies at the tool-call boundary in this sandbox
   — plain `nohup … &` AND `setsid nohup … & disown` were both measured dead between tool
   calls (the fifth sandbox to show it). Foreground runs on the fixed tree: **nf-game
   lib 581 passed / 0 failed**; the door family 37/0 (list above); walkreal_07000008_live
   4/4; the mp pin test (red as documented). **The full four-crate suite did NOT fit this
   window** — the standing rule is satisfied only partially; the next window must run
   `tools/test/suite.py` (foreground chunks via `--only` if the background kill persists).

## How it was verified

- `cargo test --offline --release -j2 -p nf-game --lib` — 581/0 on the fixed tree.
- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000008_live --
  --test-threads=1` — 4/4.
- The 16 door-family binaries, `--test-threads=1`, all green (37/0/1ign).
- The mp digest: the pin test on the fixed tree (tick0 equal, final moved), the baseline
  at a7afd219 rebuilt and re-pinned green, `NF_NET_HASHDBG=0-239` full-session dumps of
  both trees diffed (first differing frame 143, slot-3 body), the diag component stream
  on both trees (the bodies component named), and five probe/bisect builds (NF_NO_RADIAL,
  NF_KEEP_BIT20, 62aa121a, 15c0e34a, 82362142, 18ca71c8 — the last four clean).
- Scratch artifacts (dumps, diag outputs) live outside the repo
  (/home/z/my-project/scratch/digest_*.dump, diag_*.txt) and are NOT committed.

## What's left

1. **The mp digest cause**: the 4a155ddf^1/^2 isolation probe (both parents contain
   a7afd219), then fix the named regression or re-pin consciously with its cause. The
   divergence is one slot-3 contact resolution change (x +0.0012 / y −0.0024 at frame
   143) — small, deterministic, and movement-geometry-shaped.
2. **The full suite** on the final tree (fast runner; the totals line for the report).
3. Optional: once the cause lands, the four clean merge points above make a ready-made
   regression bracket for any future digest movement.

## Live checks for the main agent

1. (carried from round 4, now merge-render only) The Secure Room doorway on 07000008
   after ch 137: Bond should walk through the swung-open leaf's panel plane (x≈155.0)
   unobstructed — `--bond 154.0,43.1,-120.7,0,0` looking east with the door open.
2. **Ravine MP slot 3's spawn ledge** [L diag: slot 3 stands at (111.7, 41.5, −206.7)
   while the others are at y 35–37]: render `--bond 111.7,42.5,-206.7,0,0` around the
   walk's first seconds — name the face Bond grazes there; the port resolves that contact
   differently since 4a155ddf (x +0.0012 / y −0.0024 at t142), and seeing the geometry
   would name the triangle family for the fix.

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).
