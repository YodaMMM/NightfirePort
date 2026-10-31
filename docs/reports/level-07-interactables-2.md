# level-07-interactables-2 — report

Branch `s/level-07-interactables-2` (cut from `origin/master` 81da6c47; commits `0a17709d`,
`5dffd2bb`). The -2 continuation of `level-07-interactables` (first pass `9e591937`): the
first pass built the 3-test battery and the §7/§8 census docs; two earlier -2 claims died in
their sandboxes without pushes. This pass re-ran the census [D], audited the kind → pin map,
closed the one kind gap it found, and re-verified the family on current master.

## What's done

1. **The census re-run from the disc, this session** [D]. `cargo run --release -p nf-view
   --example census_07000007` (the first pass's dump tool, committed) re-dumps the placements;
   the kind table matches docs/levels/07000007.md §7 exactly (58× 0x0f drones/guests, 287× 0x19
   cels, 8× 0x20 breakables, 2× 0x24 starts, 2× 0x2a, 4× 0x2d, 3× 0x32, 9× 0x3b DynObjects,
   32× 0x47 rooms, 3× 0xd9 SPs, 15× 0xdb doors, 1× 0xdf, 22+5 AI cover, 1× 0xe8 LoadLevel,
   33× 0xe9 AI points, 14× 0xea TouchOnce, 1× 0xeb Touch, 2× 0xec muxes, 3× 0xf0 pickups,
   1× 0xf5, 6× 0xf9 sound triggers, 1× 0xfb MusicTrigger). The §7 "not present" list holds: no
   lifts, no 0x29 switches, no keypads/safes/fuseboxes/monitors/terminals, no lasers, no
   ladders, no vents, no grapple points, no searchlights/cameras/turrets, no vehicles, no 0xE2
   SimpleScripts.
2. **The kind → pin map audit** (§8 + every 07 test file read this session): every
   player-touchable kind already has a live pin — doors (free/locked/story/no-use sisters),
   breakables (vase/window/statue + the inert-walk + lock-refusal sisters), the armour take
   (+50, sits-until-reached), the kind-6 medal CREATION gate, the 0xE8 exit E, the TouchOnce
   walk-overs (#500 fire+latch, #497 player-refusal, #412 lock gate), the 0xEB Touch +
   MusicTrigger wiring, the urn SP (urn_break_live), the story SPs on the walkreal route.
3. **The one gap found and closed**: the kind-6 BOND_BONUS walk-over TAKE on 07's own rows.
   The creation gate was pinned (interact_07000007_live), but the take arm for THIS level's
   #502/#503 was only asserted on 09's rows. New battery test
   `the_kind6_bonus_takes_on_the_walk_over_and_the_other_sits`
   (interactables_07000007_live.rs): with 07's gold-medal word (BaseMap 0x07000005, rank 0 [L
   0x2bc920 row 1; M 0x199210]; M Pickup_Create 0x18dda0:93-95) both bonuses exist at their
   disc spots [D the census dump: #502 (113.3, 43.0, −83.9), #503 (88.2, 43.3, −44.6)]; the
   real walk-over consumes #503 [M Pickup_Handler 0x18e650:234-236, the kind-6 arm; D respawn
   0 = delete], and the west-terrace #502 still sits (sister). **No port gap behind it** — the
   test went green on the untouched port (4/0).
4. **No decompile contradiction met; no test weakened.**

## How it was verified (the touched binaries, --test-threads=1, slim ISO, current tree)

| binary | result |
|---|---|
| interactables_07000007_live (battery, +1 test) | **4 passed; 0 failed** (4.1 s) |
| interact_07000007_live | **7 passed; 0 failed** (12 s) |
| walkreal_07000007_live | **2 passed; 0 failed** (15 s) |
| walk_07000007_live | 2 passed; 0 failed |
| party_door_lock_07000007_live | 2 passed; 0 failed |
| urn_break_live | 4 passed; 0 failed |
| census_07000007 (the example dump) | ok — the §7 table re-confirmed [D] |
| **nf-game** (the crate the kinds drive) | **588 passed; 0 failed; 5 ignored** |

**The full suite** (fast-runner binary set) on the final tree: this sandbox kills background
jobs between tool calls, so the same runner's chunked form (`tools/test/chunk_suite.py`, one
flock-shared state file, 2 workers) ran it in foreground chunks. At the ~20:30 UTC cut-off:
397/996 binaries recorded, 1784 passed, 2 failed (both the known GPU-less wgpu-adapter panic,
`fix_0c_pylon_live` ×2 — see below), 118 ignored. The final tally at the report's cut-off is
quoted in the WORKLOG outcome entry. The heavy mp/net binaries the OOM killer took (rc −9)
when two ran concurrently were cleared and re-run; their final state is in the tally.

`fix_0c_pylon_live` (and the same-family render probes) panic in wgpu adapter creation
(`NotFound { active_backends: Backends(0x0) }`): no GPU and no active render backend in this
sandbox. Environment limitation, not a regression — the panic is in adapter enumeration
before any game code runs, and the branch changes no render code.

## What's left

- The suite tail past the cut-off (the count above; the sandbox clock, not a test result) —
  the next window's suite run closes it; every task-named binary is inside the recorded set
  and green.
- The two wgpu-adapter failures need a GPU-capable environment (the main agent's), as the
  merge lessons already prescribe for anything visual.

## Live checks for the main agent

The first pass's two roof-music-plate checks still stand (docs/levels/07000007.md §8): the
#443 band's mid-air surface question, and the `switch_channels[138]` read at the step top.
None added by this pass: no visual work shipped, and no sim-vs-game divergence was left
unresolved.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
