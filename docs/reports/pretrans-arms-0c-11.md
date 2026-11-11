# pretrans-arms-0c-11 — report

Branch `s/pretrans-arms-0c-11` = `origin/s/pretrans-arms-5` 19f30d481 merged with
`origin/master` 784ba2d99 (the merge kept both helper opt-ins — arc-landing + strip-P2K — the
west-sniper brush-past legs, and master's P2K wrapper; one conflict each in walkreal0c.rs and
walkreal_0700000c_live.rs). Task: the 0c handoff + east route green (the wire #282 ride-release
lever) and the 07000011 d3 pocket.

## Done — the 0c handoff and east route are GREEN, both ignores removed

`walk_the_whole_mission_from_the_start_to_the_0d_handoff` and
`walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side` walk to the 0d
hand-off / the east side on real input.

The ride-release lever, landed as the task's ignore messages named it:

1. **The opt-in `RIDE_RELEASE_PAST_A`** (walkreal0c.rs thread-local + `with_ride_release_past_a`;
   the handoff and eastroute tests only — the real-damage sisters and every other caller keep
   master's `d < 0.9` release byte-for-byte): the #282 ride's jump press fires when Bond's x
   passes 6.8 — past A (5.85 [D probe blk]), on the way to the wire family's natural stop
   ~1.05 past the end [L the #284 note's d_B ≈ 1.05] — instead of inside 0.9 of A. The old
   release landed Bond on the isolated 13.16 tier at (4.98, −223.29) whose east side overhangs
   the yard; the past-A release lands at (6.81, 13.21, −223.38) ON the ~13.2 band that runs
   east.
2. **The band's east hold + the SOUTH step-off** (flag-gated legs in the shared walk fn): the
   band walks east holding 13.05-13.21 ("the ledge to the arc" arrived (8.29, 13.05, −221.05);
   "the band's east hold" (9.37, 13.15, −219.83) [L the runs]), and the band is thin slivers —
   invisible to the 1.0-step probe rows, ending around x 9.5-10 — every leg that walks past its
   end falls 11 u with an eastward drift onto the yard (the measured (14.17, 2.01, −218.45) and
   (13.44, 2.01, −218.82) arrivals [L the runs]). A scratch fine probe (0.5-step down-scans,
   x 10-16, z −220..−216.5, not committed) mapped the floors: the 11.2 spine AND the arc's own
   12.1 pieces — at (9, −220) the arc floor sits 1 u below the band's south edge. The player
   step-off is therefore SOUTH: `walk_to((9.3, 12.0, −220.4).r(0.3))` — Bond drops 0.8 u onto
   the arc at (9.50, 12.36, −220.20) [L the green run].
3. From there the proven arc legs walk green: the arc west end (11.81, 12.35, −218.53), the
   ring south-west arc (22.65, 12.37, −217.36), the grab line, the #283 grab and ride east,
   the mission to the 0d hand-off [L the green run, 19.7 s].

No sim changes; no damage/health/top-up writes; the shared walk fn's new legs and release gate
are all behind the opt-in flag (lesson 7).

## What ran

- `walkreal_0700000c_live` whole: **2 passed, 0 failed, 2 ignored** (the handoff green; the two
  ignores are the pre-existing real-damage/Agent sister ledgers, untouched this round).
- `walkreal_0700000c_eastroute_live` whole: **1 passed, 0 failed, 0 ignored**.
- `walkreal_0700000c_barrelrow_live`: **1 passed, 0 failed** (the shared-prologue insurance).
- `comms_chatter_0c_live`: **1 passed**; `upward_shot_0700000c_live`: **2 passed** (the other
  walkreal0c.rs consumers, which stop before the changed legs).
- `walkreal_07000011_live` whole: **3 passed, 0 failed, 1 ignored** — the d3 00agent pocket
  ledger stands exactly as ffe3ed6a5 left it.
- `nf-game --lib`: **782 passed, 0 failed, 4 ignored**.

Merge-side casualty, attributed: `guard1_rounds_0700000c_live` ran 1 passed + 1 FAILED —
`guard1_rounds_at_the_ring_real_health_fight` died at tick 9710 (14 hits for 100.0; the dealers
are the deck snipers: #19 48.0 in 3 hits at 50 u, #1 27.6 in 1 hit at 2.4 u — the sight-moved
fights class). Its walk stops at the ladder base / patrol ring, BEFORE every leg this task
changed, and both my flags are off for it — the red comes from the merge's sight content
(master's drone.rs/hurt.rs/world.rs into the pretrans-arms-5 base), not this task's edits. The
-5 report never ran this binary; -4 ran it green on the pre-sight tree. Next round should
re-pin it with the sight-era stand law (the same class the los-mask-repins rounds repaired).

The full four-crate suite did not run: this sandbox kills every background job between tool
calls (the 12-2/-4/-5 reports recorded the same wall), and the foreground per-binary chunks
above filled the window.

## What's left

- **The 07000011 d3 pocket** (the 00agent sister, the ffe3ed6a5 ledger) stands unchanged — the
  0c ride-release work filled the window. Its ignore message carries the measured ledger (Bond
  reaches the 23.0 strip east end at 22.0 hp; the 18.9 pocket #18/#19/#26/#36 finish him in the
  <3.0-u free-hit band [M 0x179740:113-115]) and the two next levers: thin the pocket before it
  wakes (a stand from the 23.0 strip's north window on the #18/#19 pair filing up the
  connector), or diet the upstream 27.1-strip crossfire (29.0 → 22.0 across its loot) so the
  descent starts with the ~40 the pre-fix tree had.
- The guard1_rounds real-health re-pin above.
- The full suite on a sandbox that lets background jobs live.

## Live checks for the main agent

None this round — the ride-release landing and the band/arc floors are all harness-measured
([L] runs + the scratch probe), and the walk repairs are player-shaped without a PCSX2 poke.
(Carry-over from -4, still open: the fresh-boot gdbwatch of where the game's own #282 ride
drops Bond, and whether a player walks east off that landing onto the arc.)

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
