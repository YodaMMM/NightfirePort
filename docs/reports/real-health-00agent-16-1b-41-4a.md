# real-health-00agent-16-1b-41-4a — report (2026-11-04)

**DONE in the task's own shape: 4a and 41 are green as loaded; 16 and 1b are ignored with a
measured reason and the next lever** (the same shape the driver's done line allows). Every
`walkreal_0700*_live` binary ran whole this session, 0 failures; the four `-p` packages' totals
are below.

Branch `s/real-health-00agent-16-1b-41-4a`, cut from origin/master ecfaca48 in round 1
(commits c392554a + c05b99a2 — the four sisters wired and measured), then this round merged
origin/master 0902d3bc (merge commit 2d0528e2, clean) and re-verified everything on the merged
tree. Master since the cut brought: the 16 d2 ignore's south-gallery law with four measured
levers (real-health-agent-16-1b-2), the guard-side `glass_pass: false` bullet field and
`world.rs segment_flight_scoped` (level-08-00agent-hind's deviation), and trigger-hittest-segment.
**None of it moved the two red ledgers** — re-measured bit-identical (below).

## The sisters (each rides its file's `*_at(difficulty)` body — same route, same end, real input)

All four bodies already assert `sim.drones.difficulty == difficulty` at load, so the d3 calls
carry the assertion for free ([M] the four test files, this branch):

- `walkreal_0700004a_live.rs` `ride_the_lift_from_the_spawn_and_walk_into_the_lobby_real_damage_00agent`
  → d3. **GREEN AS LOADED** — zero damage taken, same as its d2 sister. 4a's pain row is
  Plr_DMod_Hard 1.0 (the 0x11..=0x13|0x4a group) [M Player_HandlePain 0x1902c8; 0x1d3a80 case
  0x700004a]; the lobby guards' hit chance by Accuracy Hard [M DroneWeap_DoBulletAccuracy
  0x179740:74-88] — the route's own cover holds.
- `walkreal_07000041_live.rs` — **TWO d3 sisters, both GREEN AS LOADED**:
  `walk_from_the_plate_to_the_tether_crossing_and_the_levels_end_real_damage_00agent` and
  `the_plate_first_walk_real_damage_00agent`. 0x7000041 has no ReadTuningVars case, so the
  static default rows hold: pain Plr_DMod_Hard 1.25 [M 0x1d3a80 default; L the ELF .data
  initialisers]; the crossing guards still land nothing on the walked lines.
- `walkreal_07000016_live.rs` `walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent`
  → d3. **IGNORED with the measured ledger** (below).
- `walkreal_0700001b_live.rs` `walk_arrival_to_drakes_defeat_on_real_health_00agent` → d3.
  **IGNORED with the measured ledger** (below). The body's Plr_DMod row assert was made
  d3-aware in round 1 (Hard 1.0 [M 0x1d3a80 case 0x700001b]).

No helper changes at all this task: everything rides each file's own `*_at(difficulty)` body
and the existing `with_real_damage` / `with_real_damage_head_hunting` opt-ins. No top-ups, no
health/ammo/channel writes, no new RNG draws.

## The two measured red ledgers (re-measured this session on the master-merged tree — identical)

**16 — dies at tick 1141 at (12.12, 0.93, −88.14)** (the corridor approach, at the ramp foot):

- 20 hits for 100.0: #2 skin 0x50000b2 AR 47.5 in 7, #0 0x500008c 17.5 in 5, #5 0x50000b2
  14.0 in 2, #3 0x500008c 10.5 in 3, #4 0x50000b1 7.0 in 2. Four of the five shooters stand at
  y 6.7 above the approach — the same **south-gallery row** the d2 round's law names
  (real-health-agent-16-1b-2: four AR guards 0x500008c/0x50000b2/0x50000b1 patrolling
  z −102..−112 at y 6.7, first landed round tick 455, above the plain shooters window's dy 6.0
  so the ramp-foot fights trade with the close ground pair while the gallery plinks free).
  Accuracy Hard 1.3 [M 0x179740:74-88] and pain 1.0 make the same geometry lethal ~4× earlier
  than the d2 run's 86-health bleed.
- Next lever (unchanged from the d2 round's four measured levers): the fights must take the
  gallery first from the ring's east end (lever (b) worked mechanically — the fights DID kill
  #5/#1/#3 1v1 — but the ring trades cost 44 and left 7.6 for the chasers), i.e. a
  cover-shaped gallery approach, an armour pickup before the ramp foot, or a live PCSX2 scout
  of the gallery's foot access. Then the 05-07 d3 stands (spawn pre-shoots, head-hunting,
  reload_full) on top.

**1b — dies at tick 8426 at (12.26, 5.83, 0.27)** (the Drake stand):

- 26 hits for 100.0, 0 return-fire fights: #19 skin 0x50000ba weapon 0x30 = 20.0 in ONE hit
  (Drake's 0x30 at −20.0 Hard [M 0x1d3a80 case 0x700001b: Hard 0x3f800000]) and the wave
  ring's 0x6a at −5.0: #9 15.0/2, #6 10.0/4, #7 10.0/3, #5 10.0/4 — the six 0x5000078
  astronauts at 7.9-10.0 u outpace Bond's dps exactly as the d2 round measured (their ledger
  said the same: the wave ring finishes him; the dry-window lever measured worse and was
  reverted).
- Next lever: the d2 stand work first (real-health-agent-16-1b-2's wave-ring notes — Bond's
  dps/reload cadence and the waves' sight-lines), then the d3 pressure.

## What was run (this session, merged tree 2d0528e2, `--test-threads=1`, slim ISO)

The driver's one-line suite command does not fit the 10-minute tool budget here, so it ran as
foreground chunks (the killed background attempt cost nothing — cargo resumes from target/):

- `nf-game`: **1047 passed / 0 failed** (102 test binaries).
- `nf-formats` + `nf-audio`: **125 passed / 3 failed** — exactly the 3 allowed slim-ISO
  `disc_golden` (bigf header/tables/directory).
- `nf-view` — **every `walkreal_0700*_live` binary, all 26, 0 failures**: 01: 8, 02: 3+1i,
  03: 6+1i, 04: 10, 05: 14, 06: 6+1i, 07: 5, 08: 5+1i, 09: 8, 0a: 12+3i, 0b: 7, 0c: 1,
  0c_barrelrow: 1, 0c_eastroute: 3, 0d: 1+1i, 11: 3+1i, 11_route: 1, 12: 3, 13: 3, 14: 9+1i,
  15: 5+2i, **16: 14+2i, 1b: 17+2i, 41: 15, 4a: 4** (my four files' binaries include the d3
  sisters), 4a_satchel: 2. Total 166 passed, 16 ignored.
- **NOT run**: nf-view's remaining non-walkreal test binaries (~900 files: the golden/fix_*/
  probe_* sets). The sandbox disk cannot hold their artifacts — the 05-07 round measured
  ~12 GB of release-test artifacts against 9.9 GB of disk (three disk-OOMs); this sandbox sits
  at 3.6 GB free with the ISO and work/ in place. The chunks that fit all ran green above.

## What's left

Nothing for this task's done shape. The open work the two ignores point at is the 16/1b
approach/wave-ring fight design (the real-health-agent-16-1b-2 round 2 continuation), then
re-flying these d3 sisters on top of it.

## Live checks for the main agent

1. 16's gallery: at 00 Agent, from the ramp foot (≈(12, 0.9, −88)) gdbwatch
   `DroneWeap_DoBulletAccuracy` 0x179740 on #2 (skin 0x50000b2) — the pick should read the Hard
   row (×1.3) while the guard plinks from the gallery; and a screenshot of the station's west
   side to settle whether the gallery is foot-reachable (the d2 round's open question — the
   player answer this route is missing).
2. 1b's Drake: gdbwatch the 0x30 damage row on #19's first landed round — the port reads −20.0
   at Hard [M 0x1d3a80 case 0x700001b]; one live read would pin the whole 20.0 single hit.

## Usage

not visible to me (no token/credit/context-window figures are exposed to this session; nothing
in the sandbox reports them).
