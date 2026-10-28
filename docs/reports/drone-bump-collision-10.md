# drone-bump-collision-10 — the guard push branch merged and the two merge-reds healed

Branch `s/drone-bump-collision-10` (from `origin/s/drone-bump-collision-9` 2498dc78, merged
`origin/master` 9190eaa9 — clean, no conflicts). Files changed:
`crates/nf-view/tests/walkreal_07000001_live.rs`, `crates/nf-view/tests/common/walker.rs`,
`crates/nf-view/tests/probe_07_route.rs`, new `crates/nf-view/examples/probe_07_rimobj.rs`.
No `crates/nf-game` src changes — the port matches the decompile at every step the walks take;
both reds were walk/route problems the shift exposed.

## 1. walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` — green again (4 passed / 0 failed, 215 s)

The merged tree shifts the shared-RNG stream at level start: master's vision arms set
`+0x4F8 |= 0x10000` on many state ENTERs [M 0x16a2d0:15, 0x16a3e0:31, 0x173ac0:12, 0x15ba68:24-25
— "the 01 starting guard's sight ack lands DURING his scripted walk because of this bit"], every
later first-sight roll lands differently, and by the stairwell the run differs from the -8 tree's:

- **What the failing step actually was** (the give-up dumps + the contact-settle diagnostics in
  the run log): the contact parks at route_idx 27 of 37 at (−28.3, 6.3, 14.3) — the stairwell
  head's south gap hosts an armed stepper, and the same presence wedges BOTH walkers (the
  contact mid-route, Bond's descent line repelled east by the crowd-aware steer until the
  −14.1 hall door). Bond never descended; the plate assert at :416 failed with him at the
  pocket.
- **The game's move at that step**: a player clears the stairs before descending — the kills
  are the real input. The walk now runs the SAME fight the recovery loop already ran (the
  started-armed-guard-within-14-of-the-contact filter, `w.shoot`, the resupply retries — the
  game's own hide dance `NDrone2_FindRunToPoint` 0x15168 is the unported half), re-derived to
  run BEFORE the descent [S the order choice; the filter itself is the test's existing,
  `NDrone2_NotNearArmedDrone` 0x177640-backed one]. Nothing was disarmed and no assert was
  weakened — the guards die through the real trigger path.
- **The mid-stairwell double doors** (−33.1, 6.0/7.3, 9.0, kind 2, ch_open 0, ch_lock 0
  [D the stuck dump]) auto-close 360 ticks after the contact's route opens them [M Door_Update's
  auto-close scan; the use_door doc], so Bond arrives to shut leaves. The game's player route
  through a closed idle ch_open-0 unlocked door is the E press [M the opener census:
  `Player_Activate` 0x1a5170:100-129 → `Door_Activate` 0x135be0 is the ONLY player route —
  `Door_ReactToHit` 0x135980 reverses only moving leaves, no contact opens a closed idle one].
  The walk presses E at the landing (a `Wp::Use` leg, the same shape probe_07_route's
  "E the hall door" uses), stand point (−31.6, 6.4, 9.0) — on Bond's side of the leaves'
  ~1.29-radius world spheres, where the E's head+dir·1.0 reach covers the leaf centre 1.66 away
  [M 0x1a5170:100-108].
- **The walker's new blocker-door arm** (`common/walker.rs`): the stuck handler's door-E filter
  missed this approach (its world-segment-to-door heuristic is angle-dependent), so a second arm
  fires the E when Bond is wedged with a closed unlocked door standing on the blocked walk line
  (point-to-segment < 1.2). Cited in place.
- **Double Cross Part 1 stays finishable**: the test ends on the level exit — 4 passed / 0
  failed includes the escort, the AND and the exit asserts that follow the plate.

## 2. probe_07_route's post-E rim — the object identified [D] and the route re-anchored (1 passed / 0 failed, 12.8 s)

New assertion-free probe `examples/probe_07_rimobj.rs` dumps the main-map instances whose world
AABB overlaps the wedge band [D]:

- **`OBJECT_ChestOfDrawsSmall_6b`** — world AABB x 91.50..92.32, y 42.89..44.00,
  z −46.39..−45.49 (map 47, object #217, hash 0xffffffff = anonymous by-index, params
  [(0,0),(1,1)], flags 0x8019), and its twin **`_6a`** x 90.68..91.50 (object #185).
  The walkable rim at y ≈ 43.215 the -9 capsule walk measured is the pair's east-face collision
  edge (the model top is 44.00; the capsule grounds on the mesh's lower lip).
- **`OBJECT_BookCase`** — x 86.18..91.38, y 43.00..46.20, z −52.75..−48.26 (object #262) —
  seals the hall's south-west; the first post-E leg's westward line from the door dies on its
  north-east corner region.
- The old MANUAL post-E legs hugged the pair's east face at 0.18 (the x 92.5 line vs the face at
  92.32) — under the 0.45 capsule — and the (92.6, −47.5) corner leg wedged on the recess jamb.
  A player walks the gap between the chests and the plate box on the hall's own lanes, which is
  what the nav web encodes (the same web walkreal_07's green crossing walks): the five post-E
  anchors are unchanged in destination but now `walk_nav_to`, and the whole route — E on #244,
  north-west corner, north past the plate's west edge, the middle band, #251's south side,
  through #251 — completes without the wedge. The ch48/ch130/door-251 prints are in the run
  output for the next route decision (ch 0x48 latches during the post-E legs — the nav crosses
  the #412 box; the probe records it, nothing asserts it).

This closes the -9 report's live check 1 (the rim object's identity) from the disc side; live
check 2 (#244's E from the box centre on PCSX2) stays open for the main agent — the port side
is pinned green by `party_door_lock_07000007_live` 2/0.

## What ran (all green, `--test-threads=1`, NF_ISO set)

- The two task items: `walkreal_07000001_live` 4/0 (215 s), `probe_07_route` 1/0 (12.8 s).
- The walker.rs users around the changed areas: `walkreal_07000007_live` 2/0,
  `party_door_lock_07000007_live` 2/0, `probe_07_hover` 1/0, `cutscene_timer_pause_live` 1/0,
  `walkreal_0700000d_live` 1/0, `walk_0700000d_live` 3/0, `patrol_walk_0c_0d_live` 2/0.
- `tools/test/chunk_suite.py` on this tree: build ok (892 test binaries), `nf_view` 96/0
  recorded, cumulative 3/892 binaries, 100 passed / 0 failed.
- **The full-suite total line could not be produced in this window**: 892 binaries with a
  ~25-minute rebuild after the walker.rs edit (it relinks every nf-view test binary), in a
  sandbox that kills background jobs between tool calls, inside the 2-hour budget. The chunk
  state file (`/home/z/my-project/suite_state10.json`) carries the progress; the next window
  runs `chunk_suite.py` until ALL DONE on this tree and quotes the summed tally.

## What's left

- The full-suite total line on the final tree (the chunk runner state file pattern above).
- The probe's ch-0x48 note: if the story plate must stay un-fired for #251's route, the middle
  band legs need one more west-shifted anchor pair — the prints to decide it are in the run.
- The -9 report's live check 2 (PCSX2 #244 use from the box centre) for the main agent.

## Usage

not visible to me

## ntfy failed

All three fallbacks returned the server-side daily quota error (HTTP 429):

- `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
  `{"code":42908,"http":429,"error":"limit reached: daily message quota reached; increase your limits with a paid plan, see https://ntfy.sh","link":"https://ntfy.sh/docs/publish/#limitations"}`
- `python3 -c "import urllib.request as u; ..."` →
  `urllib.error.HTTPError: HTTP Error 429: Too Many Requests`
- `wget -q -O- --post-data "drone-bump-collision-10: done" https://ntfy.sh/nightfire-yoda-8k3x` →
  (empty output, exit 0 mask over the same 429 — no body returned)
