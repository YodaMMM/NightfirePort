# script-break-drain — report

Branch `s/script-break-drain` from `origin/master` 22d5836b7. **The task's gap is closed:
the script-created breakables' single drain path is confirmed, the dead parallel queue is
removed, the disc census is pinned, and both census levels are pinned through the real
fire path** — including the driver's ride-while-moving pin.

## 1. How the windows break on master today (the finding the task asked for first)

The queue record (`done/30-port-map-scripts.md`) is STALE: master already drains the 0x20
arm — commit `372eafefd` ("995-gondola-windows-08", merged) added, in ONE commit, both the
`Objects::pending_breaks` push (nis.rs) AND the working per-tick sync (drone.rs:20855,
after the scripts walk, the same post-walk section where the pickup queue drains):
`retain_script_panes` + `script_pane` upsert every live `Obj::Entity { kind: 0x20 }` each
tick, with `Break_Create` 0x12c230's kind/health/filter taken from the op-7 params
([0]/[1]/[2] → kind / (f32)(i32) health / `~filter` [M 0x1d8d78:80-87 → 0x12c230:21-35,
disasm 0x12c2a8-0x12c2c4]) and the bounds re-derived from the entity's LIVE matrix every
tick (the game keys obj+0x90 per `Script_Interp` 0x1d9a08:94-112; the bullet tests the pane's
current local space [M `Collide_Intersect` 0x1ebaa8:157-177]). The push-keyed
`pending_breaks` list was never read by anything — dead code from birth, with comments
claiming a drain that lives elsewhere. **The routes overlapped only on paper: the queue
never created anything; the scan did all the work.**

## 2. The single path (the change)

- `nis.rs`: the dead `pending_breaks` field, the `PendingBreak` struct and the 0x20 arm's
  push are REMOVED; the arm's comment now cites the actual drain (the per-tick sync) and the
  game flow. The upsert keyed on the object index keeps the reuse path from adding a second
  breakable [M 0x1d8a68-0x1d8a80 — a same-class stream object is re-pointed, `Break_Create`
  runs only on the create path], and the pane's health state survives a re-point.
- `drone.rs`: the sync's comment now states it IS the arm's drain and the single path
  (behaviour untouched — the diff there is the comment).
- Zero behaviour delta: the removed push wrote a list nothing read. Every test binary that
  touches the surface ran green (§5).

## 3. The disc census [D] (new test `nf-formats/tests/script_break_census.rs`)

Every archive on the disc that parses as a level bin (99 .bin archives — the SP levels, the
MP maps and the loadable packs reached through the kind-0xC records [M `LoadableLoad`
0x1d69e8]), every kind-7 NIS script, every stream decoded: **exactly 34 obj_type-0x20
creates, in exactly two levels**:

| level | script | streams | params |
| --- | --- | --- | --- |
| 07000008.bin | 0x0600004C (the gondola ride) | 7-18, 23-26 (16) | [2,1,0] all; every pane parented to stream 2 (the carriage) |
| 07000012.bin | 0x06000530 (the sky-bridge intro, SP #660, mode 64 preloaded) | 10-27 (18) | [2,1,0] x16; **[1,1,0] on streams 15 and 26 (kind 1)** |

The test pins the full list (counts, script hashes, stream sets, parent, params). The
gondola was NOT the only instance — the census's second level is pinned live (§4).

## 4. The real-path pins

- **07000008, the moved position** (`gondola_windows_live::
  a_pane_shot_while_the_ride_moves_breaks_at_its_moved_position`): the ride boarded through
  the real plate/mux chain, the pane tracked from registration, the aim and trigger rode the
  real input at the pane's LIVE centre, the fire held until the carriage had descended — the
  pane was killed **19.20 u from its first-seen position** (y 28.27 → 23.36), the kill
  registered for that pane's object as (kind 2, sound 0x2E) [M `Break_Kill` 0x12b970:64-76],
  the kill-tick bullet flight crosses the pane's MOVED bounds (`Breaks::seg_box_t` over the
  last-synced row — the game tests the segment in the pane's current local space [M
  0x1ebaa8:157-177]), and the row + entity are gone (the kill's deletion [M 0x12b970:60]).
- **07000012, the second census level** (new `skybridge_panes_live.rs`): the intro's panes
  register at level start — **17 of the census's 18** (stream 12's glist carries no collision
  shape; a shapeless pane never registers — the game's own pane would be unhittable and its
  kill silent [M 0x12b970:61-63]) — 15 kind-2 + the two kind-1 (streams 15/26), health 1.0,
  filter 0 each. Bond walks to the bridge on real input and a converged real trigger kills a
  kind-2 pane: (2, 0x2E), the row dropped, the entity deleted.

## 5. What ran (all green, final tree)

| binary | result |
| --- | --- |
| `nf-game` lib | **686 passed / 0 failed / 5 ignored** |
| `walkreal_07000008_live` whole | 6 / 0 (99.3 s) |
| `gondola_windows_live` whole | 4 / 0 (the moved-position pin included) |
| `skybridge_panes_live` (new) | 1 / 0 |
| `urn_break_live` | 4 / 0 |
| `break_debris_live` | 2 / 0 |
| `broken_object_bark_live` | 4 / 0 |
| `interact_07000001_live` | 10 / 0 |
| `interact_07000009_live` | 16 / 0 |
| `interactables_07000012_live` | 12 / 0 |
| `interactables_07000013_live` | 10 / 0 |
| `nis_footsteps_live` | 1 / 0 |
| `nis_handback_census` | 1 / 0 / 1 ignored |
| `nis_skip_handback` | 2 / 0 |
| `nf-formats` `gondola_windows_script` | 2 / 0 |
| `nf-formats` `script_break_census` (new) | 1 / 0 |

**The four-crate full suite did not fit the window** (the sandbox kills background jobs
between tool calls; the foreground batches above cover every binary that touches the changed
code — the diff is dead-code removal plus comments, so every other binary is behaviour-
identical by construction). The next round can run
`NF_ISO=… python3 tools/test/chunk_suite.py` to completion.

## 6. What's left

Nothing on the task's list. Carry-overs from 995 (not this task's scope): the render check
of a pane vanishing mid-ride (the main agent's), and the laser-vs-window live check.

## Live checks for the main agent

None new — the pins are sim-side and complete. 995's three live checks (the gondola pieces
question, the laser drop, the 09 roof-glass geometry) still stand.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (nothing to copy).
