# Searchlight late spots — the creation question settled (GAPS G-036 item 1)

G-036's open item: at the original snapshot only 2 kind-2 spots exist and 1 of 7 heads is
linked on 07000005; if the game created the other 5 spots later (spawner or script), their
create-walks would backfill the remaining heads. **They are never created** — the disc carries
no placements for them and no code path creates searchlight parts outside the load-time
placement hook. The port already matches the game; this doc pins the negative proof and the
consequences.

## The disc carries only 2 spots for 7 heads [D]

Every class-0x22 placement (flags byte 0xDE) across **all** archives in FILES.BIN (the whole
`.bin` sweep, `nf_formats::level::Level::parse` per archive):

| archive | heads (kind 1) | spots (kind 2) | discs (0) | beams (3) | bulbs (4) |
|---|---|---|---|---|---|
| 07000005.bin (map 50) | 7 | **2** | 7 | 6 | 7 |
| 07000006.bin | 4 | 4 | 4 | 4 | 4 |
| 07000041.bin | 1 | 1 | 1 | 1 | 1 |

No other archive holds any 0xDE placement. The 29 on 07000005 match the snapshot's 29 class-0x22
objects one for one [L ee_start_05.bin — docs/atlas/searchlight.md], so nothing extra exists at
the start either. The 2 spots pair by placement order: head pl 27 (−57.0, 37.8, −60.0) with spot
pl 31 (−57.8, 36.6, −59.7), created **after** the head → its create-walk sees the head in place
and backfills `head.links[2]` [M 0x1cb6c0–0x1cb798 branch 4b] — the snapshot's one linked head
[L]. Spot pl 33 (61.4, 33.1, 106.6) sits ~1.5 units from head pl 230 (62.2, 34.4, 107.0) but is
created **before** it; its own kind-2 epilogue moves it to pos − 10000
[M 0x1cb8a0–0x1cb8cc] before the head's create, so the head's 3-unit walk measures ~17000 units
and never links — the measured orphan [L spot 016e8ec0, docs/atlas/searchlight.md].

## No code creates searchlight parts after load [M]

- `Searchlight_Create` 0x1cb4f8 has exactly one caller in the whole decompose:
  `parsemap_create_dynamic_objects` 0x1d04f0 (the placement hook; grep over the ACTION.ELF and
  DRIVING.ELF decompiles — DRIVING.ELF has no searchlight code and no `control_create_object`
  callers at all).
- The dynamic placement walk `parsemap_block_map_data_dynamic` 0x1d1b68 runs twice per level
  load and never later: `parsemap_parsemap` 0x1d1cf0:65 (the file-parse end, param 0) and
  `ResetMap_Load` 0x1d6360:146 (after the cels link, param 1 — the checkpoint restart re-runs
  the same pass). It skips a placement when `flags & 0xa000 != 0` or when the cel/entity
  lookup fails [M 0x1d1b68] — 07000005's 29 searchlight placements carry flags exactly
  0x000000DE [D], so both passes create the same 29 parts.
- The script entity creator `Script_CreateEntity` 0x1d8968 dispatches on the script type:
  0x29 Switch, 0xe1 Destroy, 0xea/0xeb Trigger_Touch(Once), 0xf0 Pickup, 0x20 Break, 0x2d
  Ladder (script type 0x22 maps here), 0x3e AIVolume, 0xf2 unlink — **no class-0x22 creation**
  [M the dispatch, 0x1d8a64–0x1d8d54].
- `DroneSpawner_SpawnDrone` 0x137cb0 spawns drones only [M]; no other spawner family exists in
  the index.

## Consequences (the game's own behaviour, which the port reproduces)

- The 6 heads without spots (placements 194, 199, 205, 210, 230, 261 [D]) are permanent
  orphans: their `links[2]` stay 0 for the whole level (Update never writes links except the
  kill clear [M 0x1cb900]).
- The arm test is spot-only: the 0→1 transition reads `links[2]`'s position and the whole
  beam/ray block is skipped when it is 0 [M 0x1cbf14–0x1cbf54, 0x1cbdb0 `beqz $s1`] — an
  unlinked head sweeps (state 0) forever, can never track (state 1) and never raises its alarm
  channel, no matter where Bond stands.
- The port's `Searchlights::create` reproduces the census, the 3-unit create-walk, the
  kind-slot pairing and the −10000 orphan rule (`crates/nf-game/src/searchlight.rs`, citing the
  same addresses), so the port starts and stays at exactly 1 linked head [L snapshot] — the
  tests pin this; `searchlight-tracks-bond`'s removal stands (the six heads cannot track in the
  game either).

## Live checks for the main agent

1. **No late creation** — fresh boot into 07000005, breakpoint `Searchlight_Create` 0x001cb4f8
   with a hit counter. Expected: exactly 29 hits, all during the loading screen; zero hits
   after the first play frame for as long as the session runs (a 30th hit would falsify this
   doc and reopen G-036).
2. **The orphan links stay zero** — `tools/trace/gdbwatch.py` write-watch on `links[2]`
   (inst+0x28) of head 016e03f0's instance 016e0500 [L ee_start_05.bin] across a full play
   session. Expected: no writes (only the kill path clears links, and that head's kill channel
   is 0 [D param 2 = 0]).
3. **The count stays 29** — `tools/trace/fieldprobe.py`: count the class-0x22 objects
   (`obj+0xFF == 0x22`) after 5 minutes of play. Expected: 29, same as the snapshot [L].
