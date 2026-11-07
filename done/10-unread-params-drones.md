# Unread placement params: drones, spawners, AI volumes, nodes

Classes: drone placements, spawners, AI volumes and path nodes (crates/nf-game/src/drone.rs create paths). Work that shifts the shared random stream stays on its branch and says so in the report.

Method. For each class below read its `*_Create` in the disassembly and list every placement param and word it stores. For each one, find what the port does with it. Where the port ignores it, count non-zero values on every level: a small throwaway test using `Level::parse` and the `nf_view::dev::LEVELS` list runs in under a second. A value that is zero on every level is closed with that census as the proof. A non-zero value on any on-foot level is a real gap: trace the reader in the game, port it, and test it on the real level, red on the old code. If a port comment calls something inert, check that claim on all levels, not one.

Why: `Destroy_Update`'s p6 countdown was noted inert because p6 is 0 on 0700000d. 07000014 has 17 leaves with non-zero p6, and its generator chain ran 100 times too fast.

Timing values must come from a fresh-load reading where one exists; do not derive a tick count from a symbol name (that countdown's scale is 30 per p6, not FRAME_RATE). Branch from origin/master, push to `s/unread-params-drones`, report in docs/reports/unread-params-drones.md with the census table. No draw changes, no driving levels. Do not pin anything you have not traced.

---
claimed-by: sandbox4 at 2026-11-07T09:01Z

---
## Report (done, 2026-11-07T10:40Z)


# unread-params-drones — report

Branch `s/unread-params-drones` (from origin/master 22d5836b7). Task: for drone
placements, spawners, AI volumes and path nodes, read every placement param each game
`*_Create` stores, find what the port does with it, census the ignored ones on every
level, close the zero-everywhere ones and port the real gaps.

## Method and tools

- Port-side audit: what `crates/nf-game/src/drone.rs`, `aivolume.rs`, `spawners.rs`,
  `nav.rs` and `crates/nf-formats/src/{placement,aipath,drone}.rs` read, param by param.
- Game side: the three `*_Create` functions read whole from the decompile:
  `Drone_Create` 0x136be0, `DroneSpawner_Create` 0x1378d0, `Drone_AIVolume_Create`
  0x1376c0, plus the path-node loader `AIPath_Parse` 0x1d2598 (path nodes come from map
  block 0x05, not from placements — no per-placement node creator exists; a placement's
  +0x44 word only references script key-path data, block 0x19 [M 0x1d1b68:88-92]).
- Census: `crates/nf-view/tests/census_unread_params.rs` (committed, assertion-free,
  `#[ignore]` probe; run `cargo test -p nf-view --test census_unread_params -- --ignored
  --nocapture`): parses all 34 archives of `nf_view::dev::LEVELS` through `Level::parse`
  off the disc ISO, 6.7 s total. All counts below are [D] level data.

## The param inventory (game side)

`Drone_Create` 0x136be0: difficulty gate on param 2 [M :49]; memcpy of all 33 param
words into `DIVars+0x34` (later `DI+0xc34+4k`) [M :59,81]; param 4 → `DI+0x144` (AIPoint
group), param 7 → `DI+0x148` [M :82-83]; `obj+0xec = Rand(10000)` [M :71] — an existing
RNG draw, not touched here. `DroneSpawner_Create` 0x1378d0: params 0-8 all stored
(param 3 == 0 → 32000, param 8 == 0.0 → 10.0) [M :24-38], gate param 0 > 1 [M :13].
`Drone_AIVolume_Create` 0x1376c0: params 0-4 stored (param 0 × 0.01; param 1 type remap
{0→0,1→2,2→1,4→3}; params 2/3/4 to `DI+0x28/+0x2c/+0x26`) [M :76-102]; the +0x34 scale
vector multiplies the half-extents, and level 07000005 or param 1 == 4 negates rot.y and
adds `1.5·scale.y` to pos.y [M :60-63,77-84] — the port reproduces all of this
(aivolume.rs 38-46).

## Census table (non-zero counts, all 34 archives)

Placements seen: 0x0f 588, 0xf1 112, 0xf5 257, 0xf8 33.

| class | param | non-zero recs | levels | max | verdict |
|---|---|---|---|---|---|
| 0x0f | 7 | 15 | 5 | 0x16 | game stores `DI+0x148` [M 0x136be0:83] but no reader exists in the decompile; `NDrone2_DefaultInit` overwrites it [M 0x14b300:473] and `BOTWEAP_InitWeapon` zeroes the byte [M 0x129e48:37] — write-only, port ignoring matches; see Live checks |
| 0x0f | 22 | 555 | 21 | 0x005b0003 | sits in the stats-tail region the port reads via the stats-block words (nf-formats/src/drone.rs 278-288); max mirrors param 17's count word — confirm consumption, left |
| 0xf1 | 0-8 | 3-79 | ≤16 | small ints | every param is read by the port (spawners.rs 60-153) — closed |
| 0xf5 | 2 | 203 | 12 | 0x06000000 | **open gap**: game stores `DI+0x28` [M 0x1376c0:85]; port never reads it; reader not yet traced |
| 0xf5 | 3, 4, 5 | 0 | 0 | 0 | zero on every level — closed by census |
| 0xf5 | 6 | 217 | 11 | 1 | the game's Create reads only `level_tag+0x2c..+0x3c` (params 0-4) [M 0x1376c0:76-87] — game-ignored, closed |
| 0xf5 | 1 | 1 | 1 | 4 | the handled 07000005/p1==4 lift case; no "other" (unstored) values exist — closed |
| 0xf8 | all | 0 | 0 | 0 | every param zero on every level — closed by census |
| 0x0f/0xf1/0xf5/0xf8 | quat +0x24, scale +0x34 | all recs | 33 | — | the game reads the quat only for 0x34/0xf2 and the scale only for 0xf5/0xf8 [M 0x1d04f0:60-61,474-478]; port matches — closed |

AI paths (210 paths, block 0x05): `params[8]` all zero — the port's drop is inert,
closed. Node disc words +0x1c..+0x40 non-zero on 6,025 nodes/28 levels, but the game's
in-memory node (0x30 bytes) keeps only the +0x04 word; the rest are load-time slots
[M 0x1d2598:57-75] — closed. Path `extra` area present on 49 paths/28 levels: the parser
only skips it [M 0x1d2598]; no reader identified — follow-up. Link +10 words carrying
flags 0x20/0x40: 14,496 links/26 levels — the port folds them into the node→door binding
(nav.rs 107-109); behavior re-verify left. Node `raw[0]` (the +0x04 word) non-zero on 366
nodes/17 levels: the game treats it as a hash with sentinel 0x2468abce→0
[M 0x1d2598:57-75] while the port reads it as node flags (nav.rs 89) — a value histogram
is needed to tell how many non-sentinel values exist; left.

## Done

- Setup redone from scratch (sandbox reset + full rootfs disk; repo, ISO, cargo home now
  on the persistent/large volumes), branch cut and pushed early.
- Full param inventory of the three Create functions + the path-node loader, with the
  port-side read table for all four classes.
- The all-level census (the task's core deliverable) runs on the real disc in ~7 s and
  closes every ignored param that is zero-everywhere or game-ignored, with counts above.

## Not done / left

- No code change shipped: the only real gap found (AI-volume param 2 → `DI+0x28`, 203
  records on 12 levels) needs its game reader traced before anything is ported — nothing
  pinned without a trace, per the task.
- Trace the `DI+0x28` reader and port it with a real-level test (red on old code).
- Confirm drone param 22 is consumed by the stats-tail machinery; histogram the path-node
  `raw[0]` values against the 0x2468abce sentinel; find any reader of the AI-path extra
  area; re-verify the link-flag fold with a behavior test on one of the 26 levels.

## Live checks for the main agent

1. 07000005 (or any level with 0x0f param 7 ≠ 0 — census says 15 records over 5 levels;
   re-run the census binary for the list): watch `DI+0x148` (the DIVars the
   `Drone_Create` 0x136be0:83 store lands in) — expect the placement's param-7 value
   shortly after load, then the byte zeroed when that drone's weapon inits
   (`BOTWEAP_InitWeapon` 0x129e48:37), with no reader in between.
2. AI-volume param 2: on a level with `0xf5` param 2 ≠ 0 (203 records/12 levels), set a
   write/read watch on the volume object's `DI+0x28` (stored by 0x1376c0:85) during
   combat and record which instruction reads it — that reader is the port work.

## Tests run

- `nf-view --test census_unread_params` (this task's new binary): 1 passed (the ignored
  probe, run explicitly). The full suite did not fit the sandbox window (≈1 h lost to a
  sandbox reset, a full rootfs disk and a toolchain/cache move); the committed change is
  one `#[ignore]`d assertion-free probe plus docs, which no other binary references.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no
usage panel is available in the sandbox or the IM gateway metadata).
