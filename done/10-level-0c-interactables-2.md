# level-0c-interactables-2 — 0c: every player-touchable kind behaves like the game

Branch `s/level-0c-interactables-2`, from origin/master (81da6c47, batch-1031l).

## What's done

- **The census** [D]: 575 main-map placements, 34 trigger rows on 0700000c (the placements/
  triggers dumps via nf-extract). Every player-touchable kind maps to an existing real-path
  live drive with a sister — the full map is in the WORKLOG entry. Absent from the disc:
  buttons/switches (0x29), lifts, safes, keypads, monitors, vehicles, shoji [D
  docs/levels/0700000c.md §7.1 + the kind histogram]. The 0xDC skylight touchplates #275-277
  are inert by disc data (no volume, tris 0 [D triggers dump]); the twelve 0x31 fuseboxes are
  param-less and inert [D; interact…c pins the no-use].
- **The one census gap — the AMMO-kind pickup take** — closed: no 0c test drove the sniper-ammo
  pickups (#477/#520/#521 [D]). Added `the_sniper_ammo_pickup_credits_the_pool_and_the_tower_
  pair_sits` to `crates/nf-view/tests/interactables_0700000c_live.rs`: the walk-over take of
  #477 (`PICKUP_300AP (Sniper)`, p0=1 AMMO, p1=31, p2=10 [D]) drives the real path and pins the
  game's own two-tick dance [M Pickup_Handler 0x18e650 case 1 → the 0-return gate at
  joined_r0x0018ebf0 refuses the take while `Player_EquipAmmo` 0x1a8d18's fill arm has already
  written the clip; the next tick re-takes the full rounds]:
  - exactly one logged take, `Ammo { weapon: 31, rounds: 10 }` [D p1/p2; M the return feeds the
    consume gate];
  - clip[31] = 5 (the refused tick's fill; the sniper's clip word [D weapon_data 0x1F]);
  - the 300 pool (carried[8]) = 15 — the leaked 5 + the re-take's 10;
  - owned[31] stays false (case 1 has no equip arm [M]);
  - sisters: #477 and the tower pair sit before the walk, #520 sits after; the 300 pool and the
    clip slot are empty before.
- **No port code changed**: the take path already replays the decompile bit-faithfully — the
  apparently-suspicious 20-rounds-from-a-10-round-pickup is the game's own behavior, now pinned.

## How it was verified

- interactables_0700000c_live: **6 passed; 0 failed** (the 5 existing + the new AMMO take).
- walkreal_0700000c_live: **2 passed; 0 failed**.
- nf-game: **953 passed; 0 failed; 9 ignored**.
- Full suite (tools/test/chunk_suite.py — this sandbox kills background jobs between tool
  calls, so the suite ran in foreground chunks; state
  `/home/z/suite_0c_state.json`): **244 of 996 test binaries recorded; 877 passed, 2 failed,
  67 ignored**. The remaining 752 binaries are pending, not failures. Resume (next message):
  `cd /home/z/NightfirePort && source /home/z/env-fe-pages.sh && export NF_SILENT=1 CARGO_TARGET_DIR=/home/z/NightfirePort/target && python3 tools/test/chunk_suite.py /home/z/suite_0c_state.json --limit 240 --budget 12`
  repeated until ALL DONE, then the summed tally is the total.
- The 2 failed binaries are this sandbox's, not the port's:
  - `fix_0c_pylon_live` (2 tests): both panic at `fix_0c_pylon_live.rs:19` —
    `wgpu::request_adapter` fails; the sandbox has no GPU. The test renders the FULL scene
    through the shot() path and checks pixels — it needs the main agent's GPU machine.
  - `mp_online_failures`: the networked MP binary hangs/is killed here (rc -9; a direct run
    sits on `join_in_progress_from_the_menus_then_leave_and_reconnect` for 560 s+). Socket-bound;
    needs a re-run on a machine with working loopback.

## What's left

- The remaining 752 suite binaries (resume command above; the build is warm).
- Live checks for the main agent: none new — the AMMO take is a sim-side pin. The existing
  visual pins (the pylon on-screen tests) still need the GPU render pass they were written for.

## Usage

not visible to me — this environment shows no token, cost or context-window figures.
