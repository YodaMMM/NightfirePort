# guard-floor-keep-2 — the bounds push took obj+0x30, the port took pos − 0.6

Task: continue blocked/10-guard-fell-through-floor.md — with solid guard bodies (0a9d39435) a 01
guard (the ch-0xF pair's d#4, upper west floor) was pushed off and fell to y ≈ −1e6 near
(−28.7, 27.6). Branch `s/guard-floor-keep-2` from origin/master `6a4fcf4ef`. This window began
from a **fully reset sandbox** (clone, work-slim, ISO, rustup, ALSA user prefix, nf-queue — all
redone; the lost earlier attempt's walker trace and red test are rebuilt here).

## The mechanism (probe + decomp, all [M] unless marked)

- A west-band probe of 07000001's boundary walls vs the floor (scratch, rebuilt in
  `crates/nf-view/examples/probe_gfk_walls.rs`, not committed):
  the **upper west floor's north walls #94-101 carry node y 9.65 over a 9.22 floor** (floor scan:
  y 9.2 through z 16, lower floors 5.3/0.1 at z 17, **VOID from z 18** at every x −32..−20);
  the **corridor's north wall #171 (x −14.3..−30.5, z ≈ 17.1) carries node y 5.74 over a 5.3
  floor**; wall #63 is 0x400-passable and #77 0x800-door (both mask-skipped by 0x1c00, game and
  port alike). The driver's spot (−28.7, 27.6) sits past the void edge — a guard crossing z ≈ 17
  at x ≈ −28.7 and walking north in the air reaches exactly that readout.
- `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0 + `AINetwork_HitBoundry` 0x157780: the wall
  band is **[Q.y − 0.4, Q.y + lerp(post heights)]** on the y of the sphere the caller passes
  [M 0x157780:96-104].
- `NDrone2_Collision` 0x140e90:169-174 passes **obj+0x30 — the body position, y = floor +
  hip_anim** — and adds the XZ-only push back to obj+0x30 (disasm 0x1410f0-0x141134). A standing
  guard therefore carries ~1.0 of headroom above the band bottom: a hip drop (crouch,
  dodge-crouch) cannot leave the band, the wall always bounds him.
- The port passed `feet()` = **pos − 0.6**, which rides the same hip drop down. Standing margin
  at the upper west floor's north walls: 9.62 − 9.25 = **0.37**; at the corridor's north wall:
  5.74 − 5.34 = **0.40**. ANY crouch dropped the guard below Q.y − 0.4, the wall stopped
  bounding him, and his own root motion (dodge/charge) walked him out past the boundary into the
  void — `Drone_FeetOnPoint` 0x140660's ray misses, `Drone_CollisionHandler` 0x140968
  accumulates `WldGravity` unbounded, y → −1e6. That is the pushed-guard fall: Bond's solid body
  crowds the guard at the boundary (the 0x26 latch + the Jointy pair), the guard's dodge carries
  him along the wall, and in the old port every low-pose moment was unbounded.

## The fix (one line, drone.rs:22809)

`d.nav.push_out(dr.feet(), 0.4)` → **`d.nav.push_out(dr.pos, 0.4)`** — the game's exact call
(the push is XZ-only either way; only the band's y input changes). Full citation comment in
place. Commit `402226e46`.

## Verification

- `walkreal_07000001_live` whole: **8 passed, 0 failed, 1 ignored** (the documented 00-agent
  ammo ignore) — includes the rebuilt red test
  `no_live_guard_falls_through_the_floor_on_the_walk` (the whole real route asserting no live
  team-1 guard ever reads y < −20; the shared walker collects the flag every tick, prints under
  `NF_GUARD_FALL_TRACE`).
- `npc_doors_live` 6 passed; `fix_guard_misc_live` 6 passed (both mandated for drone.rs).
- More walkreal binaries, whole, one per call: **07000003 6 passed / 2 ignored (both ignores
  pre-existing in the file), 07000004 9 passed / 1 ignored, 07000005 16 passed / 1 ignored,
  07000006 6 passed / 2 ignored, 07000007 5 passed / 1 ignored, 07000009 9 passed / 0 ignored,
  0700000a 13 passed / 2 ignored, 0700000b 8 passed / 0 ignored.**
- **Two walks shift under the game-faithful bounds and were verified against pre-fix code**
  (`git checkout 1ce798b3f -- crates/nf-game/src/drone.rs`, rerun, restore):
  - `walkreal_07000002_live::walk_the_castle_all_objectives_real_health_00agent` — Bond died at
    t17475 in the bunker corridor (4 other tests in the binary pass WITH the fix). Pre-fix:
    passes.
  - `walkreal_07000008_live::walk_the_whole_mission_on_real_input` — the laser gate never wrote
    ch 134 (4 other tests pass WITH the fix). Pre-fix: passes.
  Both are walks tuned against the OLD over-bound: with P.y = pos − 0.6, walls whose node y sits
  ≈1.0-1.6 BELOW the guard's floor with ≈2.0 height still passed the top check and bounded the
  guard horizontally, where the game (P.y = pos) skips them — the old port constrained guards
  MORE than the game. The fix removes that over-bound; the two walks' fight timelines move with
  the guards' game-true positions. **Re-tuning those two walks is owed work** (walk re-routes
  for shifted fights are the established pattern — 0a9d39435 did the same).
- Not run (owed): walkreal_07000011_live (SIGKILLed twice at startup — sandbox memory cap, not a
  test failure), walkreal_0700000c and family, 070000012-16, 0700001b, 07000041/4a and the
  probe binaries. The full-suite build does not fit the window; lesson-10 selection was applied
  and the 02/08 shifts above make a pre-fix comparison mandatory for every remaining binary.

## What's left

1. The driver-stream repro (test red on the old code) did not fire in this window: d1, d2,
   d2-agent and the ignored d3 00agent run all show **0 guard-fall lines** (the d3 run dies at
   its documented AND-gate ammo wedge, tick 19800). The red test is in place and stays green;
   the fix stands on the [M] decomp evidence above, and the band-margin arithmetic names the
   exact stream-independent hole (0.37/0.40 margins vs the game's ~1.0 headroom).
2. Re-tune the two shifted walks (07000002 00agent real-health, 07000008 whole-mission).
3. Run every remaining walkreal binary whole (one per call), each with the pre-fix comparison
   first if red.
4. WORKLOG outcome entry (this branch's docs/WORKLOG.md carries the branch-cut entry).

## Live checks for the main agent

1. **Upper west floor north edge (z ≈ 16.3-16.7, x −32..−21):** crowd the ch-0xF guard against
   the boundary walls #94-99 with Bond's body and force crouch/dodge states (gdbwatch on
   obj+0x30.y and z). Expected: the game NEVER lets him cross z ≈ 16.7 even while crouched —
   obj+0x30.y ≈ 10.2 keeps him inside the band [9.25, ~12.5] and the XZ push holds his centre
   0.4 off the wall. This settles the mechanism claim (band input, not a missing wall).
2. **The crouched guard's push:** with a guard crouched against wall #94 (obj+0x30.y ≈ 9.6),
   read the push result each tick — expected non-zero XZ while his centre is inside 0.4, i.e.
   the game's push still applies at crouch height (the exact input this fix ports).
3. **07000002 bunker corridor:** dump the guards' standing positions in the t17k fight — they
   are the re-tune targets; the fixed port should place them where the game's obj+0x30 band
   puts them (the old over-bound walls 1.0-1.6 below the floor no longer hold them).

## Usage

not visible to me — this environment shows no token, context-window or cost figures for the
session (no usage readout is exposed to the worker).

## ntfy failed

All three fallbacks failed from this sandbox (the same minute a GitHub push succeeded):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28)
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." -T 20 https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 4 (network failure), no output
