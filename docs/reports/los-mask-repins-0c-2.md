# los-mask-repins-0c-2 — the 0c walks under the game's guard sight, round 3

Branch `s/los-mask-repins-0c-2` from `origin/s/los-mask-repins-0c` (974048c8a) merged with
`origin/master` (0e1371451). Task: settle the landing-shot question in the sim, take the
ledgers' next lever on the west-ledge strip crossing, and un-ignore the 0d-handoff walk, its
real-damage sister and the eastroute ground crossing.

## Done this round

1. **The landing-shot question — SETTLED, and the answer is the equipment.** A read-only
   fire-ray probe (`[0c-fire]` in `walkreal0c.rs`'s landing diag, runs on every shared-prologue
   walk) now casts Bond's REAL fire pipeline at the strip guard: the spawn-origin gate
   ([M Player_WeaponInitBullet 0x1a4bf0:212-240 — the eye for an excluded base]), the aim snap
   with its winning stops (`aim_snap_traced` — [M 0x1a4bf0:154-164's Collide_RayIntersect mask
   0x20a]), the flight world stop with the surface run/owner ids, and the drone hit boxes along
   the ray ([M the sorted list, Collide_Update 0x1ea398:216-229; Bullet_CollisionHandler
   0x12ce80; Collide_Intersect 0x1ebaa8]). Measured [L the runs on this branch]:
   - the held weapon at the #284 landing is **the Micro-Camera 0x54** (rbase 0x54, excluded
     base → the origin is the eye) — **no round spawns from it** [M scene.rs:5002 "the
     Micro-Camera 0x54 fires only from the aim bit"; docs/systems/sound-coverage.md:82 the
     base-0x54 shutter per "bullet"]. Every earlier landing-shot attempt was a shutter tap, and
     the round-2 "dry gun" telemetry (clip 1 spare 0, weapon 0x54) read the CAMERA's magazine.
     The port needs NO fix — the game's own pipeline produces the same taps.
   - the P2K (weapon 2) sits loaded at the landing: clip slot 7, carried 41, damage 3.5,
     spread 0.0, range 50 [L the probe; the walk never spends it before the landing].
   - the plain eye rays to his feet/torso/head are clear [L, confirming the 0c ledger], but the
     torso-ANCHOR ray (feet + 0.9) enters **none** of his posed hit boxes — the AimStandFire
     pose displaces the body — and the aim snap steals onto the wall behind him (the flight
     world stop at (-6.81,13.05,-219.29), surface runs 1902/1905/1906/1922/1923/1940/1951/2004,
     all owner 57). His POSED boxes are exposed instead: head +5 at (-7.26,12.88,-218.49), torso
     +4/+53, arms +20/+21 — each enters its own box with the world clear [L the probe map].
   - **the round LANDS when the shooter aims at the drawn body**: the flag-scope walk's aimed
     stand killed guard #35 from the west lane (hp -50.0, corpse true [L the runs below]).

2. **The route lever — the ledgers' lip-top path, proven green.** The opt-in flag scope
   `with_strip_p2k_shot` (walkreal0c.rs, set ONLY by the tests named below; off = master
   behaviour for every other caller) walks the ledger's west lane off the #284 drop and takes
   the aimed shot through the guard's fire-cycle pause:
   - `walk_the_whole_mission_from_the_start_to_the_0d_handoff` (difficulty 1): **GREEN** —
     "walked out in 14499 ticks; stuck 0; falls 0", objective 3 complete on the 0d arrival,
     #35 killed by the stand (the 0x1e zero-spread aimed act [M Bullet_init 0x12c5a0:139-170],
     still-gate). Real input only; no health, armour or ammo writes.
   - `walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side`: **GREEN**
     (1 passed) — the shared prologue replays with the same stand, then the ground route east.
   - The wheel-switch dead end is explained too: `switch_to_armed` (walker_b2.rs:394) accepts
     ANY ammo in hand and returns on the camera's clip 1 without cycling at all — the same
     silent no-op the round-2 fetch's "gun switch" recorded. The gun-band variant
     (`switch_to_gun`) cycles to the P2K correctly [L the run: weapon 0x2 clip 7 spare 41].

3. **The real-damage sister — the crossing walked to the #282 ride, the arc sensitivity
   ledgered.** On the real-damage tree the fights draw #35 off his post (the west-edge stands'
   gunfire [L the run — "no live guard within 3.0 of the strip post"]), so the flag scope grew a
   NO-STAND path (isolated from the plain walks): the MOVING west-lane leg (the drop spot slides
   Bond off after ~100 idle ticks [L the wheel switch's y 1.03 fall]; walk_to's turn-first phase
   stood still ~130 ticks on the fights' exit yaw and fell to y 1.96 [L]), a 12-tick settle, the
   mid-band stop at (-7.0,-218.8) (the -8.44 thin wall wedge — invisible to the 0.5-step floor
   probe — wedged the walk; its rescue arc overshot the band edge and fell to y 2.93 at
   (-4.81,-220.51) [L]), then the diagonals and the #282 ride — **all held, health 64.2
   preserved** [L the run]. The remaining blocker: the thin-floor legs AFTER the ride are
   tick-sensitive — any leg added before the #282 grab re-times it, the ride landing shifts
   ~0.02 u, and the arc ledge leg overshoots its arrival circle off the east edge (the fall to
   y 2.01 at (8.99,-222.09) [L; the pushed-green landing (4.9688,-223.3107) holds at +55 ticks,
   the shifted one falls at +114]). The sister stays IGNORED with the full ledger; the next
   round's lever is a real-damage-scoped post-ride stop NORTH of (8.0,-222.0) (the plain run's
   hold is the north side, (7.10,12.71,-222.42)).

4. **The shared-helper change is opt-in** (merge-lesson 7): `with_strip_p2k_shot` sets two
   thread-locals — `STRIP_P2K_SHOT` (the west-lane + stand choreography) and `STRIP_NO_FIGHTS`
   (walker_b2.rs — suppresses the walker's mid-leg return-fire on the thin floors; armed at the
   landing, lifted by `walker::strip_fights_resume()` after the #282 grab). Off = master
   behaviour; every other test binary compiles against its own include of walker_b2.rs
   (the thread-local lives there so the direct-include trees build).

## What ran (release, --test-threads=1, tree 29541423d)

- walkreal_0700000c_live: `walk_the_whole_mission_from_the_start_to_the_0d_handoff` **1 passed**
  (18.8 s; the real_damage sister 1 ignored with the ledger above; the agent sister stays
  ignored — its crossfire-cadence blocker is untouched this round).
- walkreal_0700000c_eastroute_live: **1 passed, 0 failed, 0 ignored** (14.8 s).
- walkreal_0700000c_barrelrow_live: **1 passed** (7.8 s).
- interact_0700000c_live: **10 passed** (16.9 s); interactables_0700000c_live: **7 passed**
  (14.4 s).
- guard1_rounds_0700000c_live **2 passed** (21.6 s), comms_chatter_0c_live **1 passed**
  (13.7 s), upward_shot_0700000c_live **2 passed** (16.4 s) — the other walkreal0c consumers.
- The fast suite (`tools/test/suite.py`) is running on this final tree in the background; see
  the addendum below for its total line (the sandbox kills plain background jobs between tool
  calls — the setsid relaunch survived this round; if it died again, the chunks above are the
  coverage).

## What's left (the next round's work)

- The real_damage sister's post-#282 arc legs (the ledger in the test's ignore attribute has
  the exact numbers and the lever).
- The Agent sister's crossfire cadence (untouched — its own ledger stands).

## Live checks for the main agent

1. [S] In the real game at 0700000c, stand on the #284 landing with the CAMERA equipped and
   press fire: confirm no round spawns (the port's evidence [M scene.rs:5002] says the
   Micro-Camera only photographs; the port's own shutter taps produced zero bullets).
2. [S] Watch the strip guard 0x5000087 from the landing after a gunshot fired elsewhere: does
   he LEAVE his post (-7.07,12.24,-218.60)? The real-damage run's target search came up empty
   at the landing after the west-edge stands fired [L the run] — the game's hearing response
   would confirm the port's AI matches.
3. [S] From the west lane (-8.4,12.4,-218.8), fire the 0x1e at the strip guard when he holds
   still: does a round land (the port kills him there on real input [L the runs])?

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## Addendum (the final tree 29541423d + the report/worklog commits through e8b5f6e5a)

- The fast suite on the final tree: the first two launches died on build errors that were races
  with this session's own mid-refactor edits, and the third on `No space left on device` (the
  session's repeated builds filled the 9.9 GB disk; 1.3 GB was freed from the stale
  target/release artifacts). The final relaunch (22:41 UTC) is building as this report is filed —
  it cannot finish its full run inside the sandbox window, so the chunks in "What ran" above are
  the coverage: every 0700000c test binary that the change touches ran green in the foreground,
  each listed with its pass count.
- The WORKLOG entry and this report are the last commits on `s/los-mask-repins-0c-2` (e8b5f6e5a
  + this addendum).
- The finish step ran twice: the first call appended only the header (the script resolves the
  report path relative to the queue dir), the second call with the absolute path attached the
  full report to the already-closed `done/30-los-mask-repins-0c-2.md`.

## ntfy failed

All three fallbacks attempted at ~22:51–22:52 UTC (2026-11-10):
- curl: `curl: (28) Connection timed out after 20002 milliseconds`
- python3: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
- wget: timed out at 25 s (exit 124, no output)
(git push to github.com worked throughout, so the block is ntfy.sh-specific — the same wall the
los-mask-repins-0c round recorded.)
