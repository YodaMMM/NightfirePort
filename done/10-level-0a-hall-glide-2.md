# 0700000a: the west hall glide height (second pass)

`the_west_hall_glide_height_matches_the_game` (crates/nf-view/tests/walkreal_0700000a_live.rs ~line 1390, ignored) still fails: the game settles at 1.1285 [L 2026-10-26]. An earlier task (level-0a-hall-glide-settle) did not make it pass. Start from that report, log Bond's root height every tick over the west hall on the real path, find which floor/step/slope rule gives the port a different settle, compare with the player ground ray and step code, fix, and un-ignore the test.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0a-hall-glide-2`, report in docs/reports/level-0a-hall-glide-2.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox7 at 2026-11-08T16:41Z

---
## Report (done, 2026-11-08T17:51Z)

# level-0a-hall-glide-2 — the west hall glide height (second pass)

Task: `the_west_hall_glide_height_matches_the_game` (walkreal_0700000a_live, ignored) still
fails — the game settles at 1.1285 [L 2026-10-26], the port at 1.0905. Start from the
level-0a-hall-glide-settle report, log Bond's root height every tick over the west hall on the
real path, find which floor/step/slope rule gives the port a different settle, compare with
the player ground ray and step code, fix, un-ignore.

**The test stays ignored this round: the round nailed WHERE the divergence lives (the walked
wall response, not any floor/step/slope rule) and landed the per-tick tooling, but the
faithful fix needs the game's live HITTEST+0x50 accumulator values — the standing live check
— because a guessed response shape already broke 21 tests once (the settle-6-b revert).**

## What the round established

1. **The per-tick root trace over the real path** (new opt-in `NF_ROOT_TRACE=1` in
   `crates/nf-view/tests/common/walker.rs` and `walker_b2.rs` — log-only, no behaviour
   change): the port's walked glide holds y = 1.0891 → 1.0905 (a one-off +0.0014 residue from
   the jamb's vertical faces) with cc 1.0347 constant; it crosses the doorway plane arcing z
   16.32 → 17.22 (t354-368) and never returns.
2. **The capsule envelope is NOT the divergence.** The two arm encodings — the walk arm
   (A = pos+0.275, B = pos+0.55−cc, r 0.55 [M disasm 0x195444/0x195464]) and the case-0/6 arm
   (A = pos+0.55, B = pos+0.275−cc, r 0.275 [M 0x194df0 case 0/6]) — span the identical
   envelope root+0.825 down to root−cc (the lowest point 0.054 above the floor at the rest).
   NF_CAPS_LOG shows the port's doorway contacts are all VERTICAL jamb faces (q y
   0.606-1.094, horizontal pushes); the 0.19-tall jamb base's top edge is never touched
   because the port's path is 0.9 u north of it when crossing its x-range.
3. **The held-forward probe is the decisive half** (`probe_held_forward_doorway_walk`,
   assertion-free, `#[ignore]`, in walkreal_0700000a_live): driving the game's own [L]
   probe walk (hold forward, no nav steering), the port WEDGES at x = −7.7540 — exactly the
   jamb face (−8.304) plus the 0.55 radius — with the z creep dying asymptotically. **The
   port's wall response is a hard projection** (the spine clamped at face + r, zero
   penetration). The game's walked wall rest is **0.011 INSIDE the plane** [L fresh boot,
   recorded in this file's corner note]: the game's move integrates unclipped [M Player_Update
   0x193600:284-295] and press-races — the per-tick collision push (the DeltaP tail
   [M 0x109da0's push arm, accumulated into BL+0x750, added back to the position at
   Player_CollisionHandler 0x1949a8]) balances against the walk input, leaving the spine
   slightly inside the face and STILL SLIDING. That slide is what hugs the jamb (the game's z
   16.29 → 16.60 through the base's x-range [L 2026-10-26]), grazes the base's top edge with
   the same capsule envelope, and takes the edge ride's upward spine pushes (v̂ = B − q points
   up, magnitude r − dist): +0.004/tick to the 1.1465 peak, the feet band holding the lifted
   root (gravity is band-gated [M 0x194df0:104]), and dy > cc+0.1 at the peak bringing the
   descent to the settled 1.1285 [L]. The port's arc never grazes, so it never climbs and
   settles at the walk's untouched 1.0905.
4. **The floor/step/slope rules check out.** `Player_FeetOnPoint` 0x194188 is detection-only
   (the ray from the root along the matrix up to HITTEST+0x30 − up·(cc+0.6); the band
   `hit+0xC ≤ cc + 0.1` sets the grounded bit 8 and writes the hit normal's y to BL+0x110);
   `Player_Collision` 0x194df0's walk case only decays BL+0x50 (×2·REC_FRAME_RATE per tick
   when banded with normal-y ≥ 0.5) and integrates pos += BL+0x50·REC_FRAME_RATE; the walk
   move itself adds through the yaw-only matrix (no y) [M, the glide-settle round's reading
   re-verified]. There is no step-up, no floor snap, no slope ride in the walked path — the
   vertical history is: collision pushes lift, the band holds, gravity pulls only above
   cc+0.1. The port has every one of these pieces; what it lacks is the game's push-RACE
   magnitude at the walls, which is exactly what the standing live check names.
5. **Why the fix is not landed blind:** the press-race tail's gates read `HITTEST+0x50` — the
   lifetime DeltaP accumulator (zero at boot [L ee_start_0700000a 0x1b78ee0], the add-back at
   Collide_Intersect 0x1ebaa8:353-373 its only writer). Whether the game's walked accumulator
   sits within the radial arm's `|q − ref| < r` gate at the doorway — and with what values —
   is the open live check from the glide-settle round. The settle-6-b round ported a guessed
   always-on tail and broke 21 tests across the suite (the y-settle class floated ~1 unit);
   the faithful port needs the measured ref values first.

## What landed

- `NF_ROOT_TRACE` (opt-in, log-only) in both shared walkers + the held-forward probe
  (`probe_held_forward_doorway_walk`, `#[ignore]`, --nocapture) — the diagnostic pair the
  next round (or the live-check round) drives with.
- The glide test's ignore note re-pinned twice with this round's measurements (the graze
  geometry + the wedge), so the next round starts from the shortlist: port the press-race
  response with the live ref values, un-ignore.
- WORKLOG entries. **No `crates/*/src` change** — the tree's walk code is byte-identical to
  the merged origin/master base (only test files and docs changed).

## How it was verified

The walkreal sweep (both shared helpers touched, so every level binary, one level per tool
call; pass/failed/ignored):

| binary | result | | binary | result |
| --- | --- | --- | --- | --- |
| walkreal_0700000a_live | 13/0/3 | | walkreal_0700000d_live | 2/0/1 |
| walkreal_07000001_live | 8/0/0 | | walkreal_07000011_live | 1/0/0 |
| walkreal_07000002_live | 3/0/1 | | walkreal_07000011_route_live | 1/0/0 |
| walkreal_07000003_live | 6/0/1 | | walkreal_07000012_live | 4/0/0 |
| walkreal_07000004_live | 10/0/0 | | walkreal_07000013_live | 3/0/1 |
| walkreal_07000005_live | 14/0/0 | | walkreal_07000014_live | 9/0/1 |
| walkreal_07000006_live | 6/0/1 | | walkreal_07000015_live | 8/0/1 |
| walkreal_07000007_live | 6/0/0 | | walkreal_07000016_live | 16/0/2 |
| walkreal_07000008_live | 6/0/0 | | walkreal_0700001b_live | 17/0/2 |
| walkreal_07000009_live | 8/0/0 | | walkreal_07000041_live | 15/0/0 |
| walkreal_0700000b_live | 7/0/0 | | walkreal_0700004a_live | 4/0/0 |
| walkreal_0700000c_live | 3/0/1 | | walkreal_0700004a_satchel_live | 2/0/0 |
| walkreal_0700000c_barrelrow_live | 1/0/0 | | walkreal_0700000c_eastroute_live | 1/0/0 |

Every binary green. 14 and 15 were SIGKILLed once each by the sandbox under the default
parallel threads and are green single-threaded (`--test-threads=1`) — the environment-kill
pattern the prior rounds' suite records also shows (an environment kill, not a test failure).

The full 4-crate suite did not fit the window again (setup + the sweep consumed it; the
sandbox also hit a full disk once mid-round, cleared by dropping stale test executables — no
repo file was touched by that). No `crates/*/src` file changed, so there is no touched-code
sweep beyond the walkreals.

## What's left (the next round's plan)

1. The live check (unchanged from glide-settle, now with the wedge evidence making it
   decisive — see below).
2. With the ref values in hand: port the press-race wall response (the accumulator + the
   radial arm's gates already exist in world.rs behind the experiment gates), keep the
   settle-6-b revert's lessons (never an always-on v̂ tail), run the full walkreal sweep +
   the settle-touching suites, un-ignore the glide test (and likely the corner sister).

## Live checks for the main agent

1. **The walk HITTEST's ref through the doorway** (decisive, unchanged from glide-settle):
   fresh boot 0700000a, probe-place Bond at (−8.0, 1.12, 16.29), walk west exactly as the
   2026-10-26 glide; per frame for ~40 frames dump `BL+0x790..0x79C` (HITTEST+0x50, the
   accumulator), `obj+0x30.y`, `BL+0x750..0x75C` (the per-tick DeltaP), and the walk's
   from/to (`BL+0x760/+0x770`). Address math: the player obj 0x1b78640 → BL = obj+0xE0
   [L ee_start_0700000a], the walk HITTEST = BL+0x740 [M 0x1910f8:40]. Expected if the
   press-race reading is right: the accumulator's walked value sits within 0.55 of the jamb
   contacts during the slide and the press ticks carry a non-zero DeltaP with +y during the
   base-edge graze.
2. **The wall rest**: at the wedge the game's spine sits 0.011 inside the face plane [L
   fresh boot] — a second probe holding forward into the jamb face (no slide) for ~30 frames,
   dumping the spine x and the face plane x per frame, pins the game's equilibrium penetration
   and the per-tick push magnitude — the numbers the port's response needs.

## Usage

not visible to me — this session's driver surface shows no token, credit, cost or
context-window figures (the IM gateway metadata carries only session/chat/trace ids).

## ntfy

(the notification result is recorded at the end of this file after the finish steps.)

---
## Report from the worker (done, 2026-11-08T17:53Z)

# level-0a-hall-glide-2 — the west hall glide height (second pass)

Task: `the_west_hall_glide_height_matches_the_game` (walkreal_0700000a_live, ignored) still
fails — the game settles at 1.1285 [L 2026-10-26], the port at 1.0905. Start from the
level-0a-hall-glide-settle report, log Bond's root height every tick over the west hall on the
real path, find which floor/step/slope rule gives the port a different settle, compare with
the player ground ray and step code, fix, un-ignore.

**The test stays ignored this round: the round nailed WHERE the divergence lives (the walked
wall response, not any floor/step/slope rule) and landed the per-tick tooling, but the
faithful fix needs the game's live HITTEST+0x50 accumulator values — the standing live check
— because a guessed response shape already broke 21 tests once (the settle-6-b revert).**

## What the round established

1. **The per-tick root trace over the real path** (new opt-in `NF_ROOT_TRACE=1` in
   `crates/nf-view/tests/common/walker.rs` and `walker_b2.rs` — log-only, no behaviour
   change): the port's walked glide holds y = 1.0891 → 1.0905 (a one-off +0.0014 residue from
   the jamb's vertical faces) with cc 1.0347 constant; it crosses the doorway plane arcing z
   16.32 → 17.22 (t354-368) and never returns.
2. **The capsule envelope is NOT the divergence.** The two arm encodings — the walk arm
   (A = pos+0.275, B = pos+0.55−cc, r 0.55 [M disasm 0x195444/0x195464]) and the case-0/6 arm
   (A = pos+0.55, B = pos+0.275−cc, r 0.275 [M 0x194df0 case 0/6]) — span the identical
   envelope root+0.825 down to root−cc (the lowest point 0.054 above the floor at the rest).
   NF_CAPS_LOG shows the port's doorway contacts are all VERTICAL jamb faces (q y
   0.606-1.094, horizontal pushes); the 0.19-tall jamb base's top edge is never touched
   because the port's path is 0.9 u north of it when crossing its x-range.
3. **The held-forward probe is the decisive half** (`probe_held_forward_doorway_walk`,
   assertion-free, `#[ignore]`, in walkreal_0700000a_live): driving the game's own [L]
   probe walk (hold forward, no nav steering), the port WEDGES at x = −7.7540 — exactly the
   jamb face (−8.304) plus the 0.55 radius — with the z creep dying asymptotically. **The
   port's wall response is a hard projection** (the spine clamped at face + r, zero
   penetration). The game's walked wall rest is **0.011 INSIDE the plane** [L fresh boot,
   recorded in this file's corner note]: the game's move integrates unclipped [M Player_Update
   0x193600:284-295] and press-races — the per-tick collision push (the DeltaP tail
   [M 0x109da0's push arm, accumulated into BL+0x750, added back to the position at
   Player_CollisionHandler 0x1949a8]) balances against the walk input, leaving the spine
   slightly inside the face and STILL SLIDING. That slide is what hugs the jamb (the game's z
   16.29 → 16.60 through the base's x-range [L 2026-10-26]), grazes the base's top edge with
   the same capsule envelope, and takes the edge ride's upward spine pushes (v̂ = B − q points
   up, magnitude r − dist): +0.004/tick to the 1.1465 peak, the feet band holding the lifted
   root (gravity is band-gated [M 0x194df0:104]), and dy > cc+0.1 at the peak bringing the
   descent to the settled 1.1285 [L]. The port's arc never grazes, so it never climbs and
   settles at the walk's untouched 1.0905.
4. **The floor/step/slope rules check out.** `Player_FeetOnPoint` 0x194188 is detection-only
   (the ray from the root along the matrix up to HITTEST+0x30 − up·(cc+0.6); the band
   `hit+0xC ≤ cc + 0.1` sets the grounded bit 8 and writes the hit normal's y to BL+0x110);
   `Player_Collision` 0x194df0's walk case only decays BL+0x50 (×2·REC_FRAME_RATE per tick
   when banded with normal-y ≥ 0.5) and integrates pos += BL+0x50·REC_FRAME_RATE; the walk
   move itself adds through the yaw-only matrix (no y) [M, the glide-settle round's reading
   re-verified]. There is no step-up, no floor snap, no slope ride in the walked path — the
   vertical history is: collision pushes lift, the band holds, gravity pulls only above
   cc+0.1. The port has every one of these pieces; what it lacks is the game's push-RACE
   magnitude at the walls, which is exactly what the standing live check names.
5. **Why the fix is not landed blind:** the press-race tail's gates read `HITTEST+0x50` — the
   lifetime DeltaP accumulator (zero at boot [L ee_start_0700000a 0x1b78ee0], the add-back at
   Collide_Intersect 0x1ebaa8:353-373 its only writer). Whether the game's walked accumulator
   sits within the radial arm's `|q − ref| < r` gate at the doorway — and with what values —
   is the open live check from the glide-settle round. The settle-6-b round ported a guessed
   always-on tail and broke 21 tests across the suite (the y-settle class floated ~1 unit);
   the faithful port needs the measured ref values first.

## What landed

- `NF_ROOT_TRACE` (opt-in, log-only) in both shared walkers + the held-forward probe
  (`probe_held_forward_doorway_walk`, `#[ignore]`, --nocapture) — the diagnostic pair the
  next round (or the live-check round) drives with.
- The glide test's ignore note re-pinned twice with this round's measurements (the graze
  geometry + the wedge), so the next round starts from the shortlist: port the press-race
  response with the live ref values, un-ignore.
- WORKLOG entries. **No `crates/*/src` change** — the tree's walk code is byte-identical to
  the merged origin/master base (only test files and docs changed).

## How it was verified

The walkreal sweep (both shared helpers touched, so every level binary, one level per tool
call; pass/failed/ignored):

| binary | result | | binary | result |
| --- | --- | --- | --- | --- |
| walkreal_0700000a_live | 13/0/3 | | walkreal_0700000d_live | 2/0/1 |
| walkreal_07000001_live | 8/0/0 | | walkreal_07000011_live | 1/0/0 |
| walkreal_07000002_live | 3/0/1 | | walkreal_07000011_route_live | 1/0/0 |
| walkreal_07000003_live | 6/0/1 | | walkreal_07000012_live | 4/0/0 |
| walkreal_07000004_live | 10/0/0 | | walkreal_07000013_live | 3/0/1 |
| walkreal_07000005_live | 14/0/0 | | walkreal_07000014_live | 9/0/1 |
| walkreal_07000006_live | 6/0/1 | | walkreal_07000015_live | 8/0/1 |
| walkreal_07000007_live | 6/0/0 | | walkreal_07000016_live | 16/0/2 |
| walkreal_07000008_live | 6/0/0 | | walkreal_0700001b_live | 17/0/2 |
| walkreal_07000009_live | 8/0/0 | | walkreal_07000041_live | 15/0/0 |
| walkreal_0700000b_live | 7/0/0 | | walkreal_0700004a_live | 4/0/0 |
| walkreal_0700000c_live | 3/0/1 | | walkreal_0700004a_satchel_live | 2/0/0 |
| walkreal_0700000c_barrelrow_live | 1/0/0 | | walkreal_0700000c_eastroute_live | 1/0/0 |

Every binary green. 14 and 15 were SIGKILLed once each by the sandbox under the default
parallel threads and are green single-threaded (`--test-threads=1`) — the environment-kill
pattern the prior rounds' suite records also shows (an environment kill, not a test failure).

The full 4-crate suite did not fit the window again (setup + the sweep consumed it; the
sandbox also hit a full disk once mid-round, cleared by dropping stale test executables — no
repo file was touched by that). No `crates/*/src` file changed, so there is no touched-code
sweep beyond the walkreals.

## What's left (the next round's plan)

1. The live check (unchanged from glide-settle, now with the wedge evidence making it
   decisive — see below).
2. With the ref values in hand: port the press-race wall response (the accumulator + the
   radial arm's gates already exist in world.rs behind the experiment gates), keep the
   settle-6-b revert's lessons (never an always-on v̂ tail), run the full walkreal sweep +
   the settle-touching suites, un-ignore the glide test (and likely the corner sister).

## Live checks for the main agent

1. **The walk HITTEST's ref through the doorway** (decisive, unchanged from glide-settle):
   fresh boot 0700000a, probe-place Bond at (−8.0, 1.12, 16.29), walk west exactly as the
   2026-10-26 glide; per frame for ~40 frames dump `BL+0x790..0x79C` (HITTEST+0x50, the
   accumulator), `obj+0x30.y`, `BL+0x750..0x75C` (the per-tick DeltaP), and the walk's
   from/to (`BL+0x760/+0x770`). Address math: the player obj 0x1b78640 → BL = obj+0xE0
   [L ee_start_0700000a], the walk HITTEST = BL+0x740 [M 0x1910f8:40]. Expected if the
   press-race reading is right: the accumulator's walked value sits within 0.55 of the jamb
   contacts during the slide and the press ticks carry a non-zero DeltaP with +y during the
   base-edge graze.
2. **The wall rest**: at the wedge the game's spine sits 0.011 inside the face plane [L
   fresh boot] — a second probe holding forward into the jamb face (no slide) for ~30 frames,
   dumping the spine x and the face plane x per frame, pins the game's equilibrium penetration
   and the per-tick push magnitude — the numbers the port's response needs.

## Usage

not visible to me — this session's driver surface shows no token, credit, cost or
context-window figures (the IM gateway metadata carries only session/chat/trace ids).

## ntfy failed

All three fallbacks failed (2026-11-08 ~17:53 UTC; the same block the level-01-guards-shut-door
round recorded from this sandbox):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-0a-hall-glide-2: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "... urllib.request ..."` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-0a-hall-glide-2: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → hangs; killed at the 25 s `timeout` with exit 124 (a second confirm run identical).
