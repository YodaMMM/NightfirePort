# level-4a-every-objective-3 — the scoped drone bullet flight vs the 08 fusebox walk

Branch `s/level-4a-every-objective-3` = `origin/s/level-4a-every-objective-2` (d5d604a0) merged
with origin/master (48696227). The -2 branch's `Bullet::update` change (the drone-fired flight
querying `segment_bullet_scoped(find_cel(pos))` + the additive world.rs probe) makes
`walkreal_07000008_live::walk_the_whole_mission_on_real_input` red at "shooting the fusebox
wires raises ch 137". This session: the repro, the isolation, and the mechanism — with the
evidence — plus what the fix needs.

## The repro and the isolation (both rerun this session)

- The merged tree: the walk reaches the fusebox, holds the converged laser aim for the full
  420 ticks, ch 137 never rises — FAILED (2 passed / 1 failed), reproducible.
- **The one-line revert experiment** (`Bullet::update`'s scoped query → the flat
  `segment_bullet`, everything else identical): the 08 walk goes GREEN (3 passed / 0 failed).
  The -2 change is the delta, confirmed by experiment.
- The 4a side: the -2 branch's own bins (the ride fire leg, the group-2 pin, the scoped-sensitive
  pins) were green on -2 per its report; the merged tree's final suite (below) re-checks them.

## Why the fusebox shot stops landing (the mechanism, with the evidence)

The fusebox contact path itself is UNCHANGED and correct on the merged tree — traced, not
guessed:

- The wire state at the fire window is identical on both trees: 1 wire, script obj43, hash
  020008b9, `wire_geo` present, the object matrix translation constant at (154.35, 43.41,
  −119.67) [probed per fire tick, temporary instrumentation, removed from the tree].
- Bond's laser (0x4E) rounds have spread 0.0 [L ee_snapshot.bin weapon_data + 0x4E·0x10C +
  0x24 = 0.0] and live ONE tick with the range-clamped segment [M the +0x1C boundary clamp,
  Bullet_update 0x12f014-0x12f02c; the record's +0x1C = 4.0 [L same dump]] — so the shot's
  reach is fixed by the muzzle pose, not by any RNG stream.
- The divergence is UPSTREAM of the shot: under the cel-scoped candidate set the guards'
  rounds fly where the flat query's walls used to kill them (the session's event probe:
  838 drone-round events inside the station window, including repeated `Player{damage 2.0}`
  hits on Bond during the approach), and the walk's upper route then gives up —
  "the station doors nav 10", "the lower landing nav 1", "by the fusebox nav 1/3" all hit
  their 3000-tick caps on the scoped tree (the STUCK dumps name a blocking face each) while
  the same nav legs arrive on the flat tree.
- The consequence at the fusebox: on the flat tree Bond fires FROM THE UPPER LEVEL (muzzle
  (152.72, 43.70, −119.66), the wires at y 43.41 — the 4.0 segment crosses the wire box);
  on the scoped tree the climb legs gave up and Bond fires from the LOWER level (bond y
  37.35, muzzle y 38.26, the segment topping at y 42.00 — 1.4 below the wire box). The shot
  physically cannot reach; ch 137 never rises.

So the port's scoped candidate set itself checks clean against the game's shape — the BFS
cap 0x7e, the stamp visit, the per-portal intersection gate [M Collide_StraddleCels 0x1eac98
decomp: the BFS from HITTEST+0x80 through `Intersect_Portal`, and Bullet_update 0x12e7b0:343
seeding `Collide_RayIntersect` with obj+0x20] vs the port's `straddle`/`ray_portal_hit` —
and the -2 change matches the game's flight rule. What the change exposes is that the 08
walk's route rides on the FLAT flight's colder station: the guards' return fire pattern is
part of the route's working state, and the scoped (game-accurate) flight makes the station
hotter than the route tolerates.

## What the fix needs (next session's first moves)

1. **The walk's fusebox approach hardened for the hotter station** — the game's player deals
   with live guards; the walk's climb legs need a retry/re-route when the first attempt gives
   up (the lenient nav skip currently strands Bond BELOW the fusebox level and the leg fires
   anyway). A pre-shot stance assert (Bond at y ≈ 43.1 beside the fusebox) turns the silent
   wrong-level shot into an honest failure while the retry lands.
2. **The AI cascade verified, not assumed**: the STUCK faces (the station doors nav 10's face
   at t 0.13, the lower landing's face at 0.996, the fusebox climb's face at 0.82) need one
   probe pass each naming the object (a live guard on the stair? a door held by a diverged
   alert?) — then either the walk routes around it or the AI side gets the fix with its own
   [M/D] cite.
3. The -2 change stays: it is the game's rule [M 0x12e7b0:343 + 0x1eb320:125 → 0x1eac98], the
   -2 branch's scoped-sensitive pins all passed on it, and the flat revert would undo a
   decompile-backed flight rule to keep a route brittle.

## What was run

- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000008_live --
  --test-threads=1`: the merged tree FAILED (2p/1F, the fusebox assert); the one-line revert
  GREEN (3p/0F); the probe runs FAILED identically (the instrumentation was temporary and is
  removed; the final tree is the clean merge state).
- `cargo build --offline --release -p nf-game`: clean after the scaffolding removal.
- The FULL suite (the fast runner) is ON the final tree as this report is written — its total
  line lands in the WORKLOG and the driver's ntfy line; the background-runner instability of
  the past two windows did not recur this time (setsid detach, alive through the window).

## Live checks for the main agent

1. The 08 station under fire: fresh boot into 07000008, walk the fusebox approach with the
   guards live — do the station's guards' rounds reach the player on the stair climbs (the
   port's scoped flight says yes; the flat port said no)? Which stair/door changes hands.
2. The fusebox shot's stance: in the game, where does the player stand to cut the wires — the
   upper level beside the box (the port's flat-tree spot) or below it? (The port's wire box
   hangs at y 43.41 with a 4.0-range laser [L weapon_data +0x4E·0x10C +0x1C]; a below-level
   shot cannot reach — so the game's cut must happen from the upper level too.)

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
