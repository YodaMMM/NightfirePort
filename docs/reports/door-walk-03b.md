# door-walk-03b — the Agent real-damage walk green; the void strips mapped

Branch `s/door-walk-03b` from `origin/s/door-walk-03` (d4972b69f), merged `origin/master`
a6665d115 (clean). Test-side only: `crates/nf-view/tests/walkreal_07000003_live.rs` and the
WORKLOG. No door, collision or game code touched; no [L] pin loosened.

## 1. The Agent walk, fixed

**The red**: `walk_the_full_route_and_complete_every_objective_real_damage_agent` — Bond died
tick 52087 at (23.39, 6.37, 11.87).

**The death ledger** (the run's own damage log, this branch, 15 s run):

| threat | weapon | distance | damage | state at the death |
| --- | --- | --- | --- | --- |
| drone #5 (skin 0x500003a) | 0x1c SPAS-12 | **2.1 u** | **59.4 in 26 hits** | alive, hp 9 |
| drone #3 (skin 0x5000038) | 0xe | **1.8 u** | 21.0 in 2 hits | alive, hp 24 (captain) |
| drone #4 | 0x0 | 3.2 u | 5.3 in 5 hits | killed true at 51861 |
| drone #11 | 0x0 | 41.9 u | 7.0 in 1 hit | killed true at 9150 |
| drone #10 | 0x0 | 31.9 u | 3.2 in 3 hits | killed true at 8303 |

Sequence: the si-0 (0x50) servant at (30.8, 6.2, −5.7) is only reachable around the east wing
— the nav's A* hops z −1.8 → +1.0 → +8.5 through the door (22.3, 5.3, 10.9) and the 0x23
plate, and the east-lawn trio met the nav mid-walk. The return fire killed #4 but the
point-blank crossfire chewed 85.6 → 52.4 inside the #4 fight, the #5 fight failed on a DRY gun
(the next line reads `weapon 0x12 clip 0 spare 0`, nav 6 "cycling weapons (0)"), and the
240-tick failed-fight cooldown covered the death volley (six pellets at 2.1 u on the death
tick). The mid-nav return fire is `stand_and_fire` — it holds position by design ("a player
returning fire on a ledge does not sidestep off it"), so the walker's keep_off 4.0 never
applies there.

**The far shooters**: #11/#10 sit past the 20 u shooters window and did plink free — but both
were already dead at the death and cost 10.2 of the 100 total. They are not the lethal
mechanism, so the 06 `stand_shooters_down` sweep was NOT added this round; if a later run
shows a far-shooter bleed, it is the ready-made lever (recent hitter within 40 u, strict gun
rays, `aimed_stand`).

**The fix** (gated `difficulty >= 2` throughout — the two green difficulty-1 walks run
byte-identical code):

1. The 00 Agent walk's resupply arms extended to Agent: before the servants' rescue, before
   each enable plate, before each servant walk. The 00 Agent ledgers document the identical
   dry-gun death one difficulty up ("the dry P99 (clip 1 spare 0) met shotgun captain #5 at
   1.6 u and died to his volley at t42315"); Agent runs the same failure mode. The resupply
   walks the covered path only (the 9-u corridor rule), so the route shape is unchanged.
2. An east-lawn saddle pre-clear before the si-0 servant walk: `walk_nav_to (23.5, 6.3, 8.4)`
   + `clear_near(9.0)` + the kill-loot pass. The saddle is nav-reachable (the red run's own
   nav 3 arrived there at +298 ticks); from it the trio is at 1.5-6.5 u where `w.shoot`'s
   keep_off 4.0 works, unlike the mid-nav stand.

**Measured** (`/tmp/agent_run2.log`, the run on the pushed tree): GREEN first try. The trio
was answered during the 0x50-plate nav with ammo flowing (#4 killed 105342, #5 105416, #3
105721), the captor-room cluster (#14/#2/#18/#19) died to the return fire at 106001-106591,
the exit lawn's #16 answered at 131396 — **health held at 85.6 from tick 9967 to the end,
133964 ticks, `falls: []`**.

## 2. The whole 03 binary + the sisters

| binary | result |
| --- | --- |
| `walkreal_07000003_live` whole | **6 passed / 0 failed / 1 ignored** (the 00 Agent test per its own ledger's ignore) |
| `walkreal_07000002_live` | 3 passed / 0 failed / 1 ignored |
| `walkreal_07000004_live` | 10 passed / 0 failed |
| `interactables_07000003_live` | 9 passed / 0 failed |

The four-crate full suite did NOT run this window (the 03 binary alone is 310 s; the window
went into the ledger reads and the sisters). The branch changes one test file — no
`crates/*/src/`, nothing under `crates/nf-view/tests/common/` — so no other binary's inputs
moved; the changed fn's callers are all in the 03 binary, which ran whole.

## 3. Does the void fall need a game fix?

**Where Bond left the geometry** (scratch probe `dw03b_voidprobe`, run and deleted; down-ray
grid x 10..17.5 step 0.75, z −28.5..−36.5 step 0.75 from y 9):

- The moat bend south of the one-way door[1] (13.96, 6.63, −30.27) has **two strips with NO
  collision at all**: **x ≈ 11.1–13.4 and x ≈ 15.6–17.5+, z ≈ −30.4..−35.6** (at z −30.75
  through −35.25 every sampled column in those x ranges rays 80 u down and hits nothing).
  Floor exists on both sides: the walkable band x ≤ 10.75 (y 5.52-5.34 — the rerouted route's
  own anchors (9.95, 5.84, −34.41) / (6.87, 5.84, −33.47) stand on it), a mid strip
  x 13.75-15.25 (y 5.25), and the south skirt z ≤ −36 (y 5.25). At z −28.5/−29.25 the east
  side is the garden wall's sloping ground (8.59 → 5.59).
- The red run's line: the A* rode the upper-floor row to its garden end
  ((14.53, 8.25, −28.98) → (16.68, 6.75, −29.00)), the wedge ladder stood Bond ON the door top
  (14.06, 9.59, −29.61), and the garden diagonal toward the west lawn crossed **x ≈ 13.0 at
  z ≈ −30.5..−31** — the last floor under that line is the mid strip's west edge (13.75,
  y 5.25); the next step west is the void. The dumps read y −2060 → −4310 → −62818 with
  `floor_below: None`.

**Could a player do the same?** Yes, by the same moves the walk used: the building's upper
floor is real walkable geometry (a player can walk the row to its garden end), and the door
top is reachable with the game's own rail/jump move (the walk's wedge ladder landed there
without any cheat). From either perch the moat bend looks like walkable ditch; stepping off
at x 11.5-13 or x 16-17.5 drops a player through the water into the void with no floor to
stop the fall. The rerouted ground route never crosses the strips (the fixed walk's `falls: []`
spans the whole 133964-tick route).

**Does it need a game fix?** The port is faithful so far as the level data goes — the strips
are the map's own geometry holes, and the port models no fall-out reset. Whether the REAL game
resets a void-fallen player is the deciding fact and needs the main agent:

## Live checks for the main agent

1. **The void strips** — in PCSX2, walk Bond off the building's upper-floor garden end or the
   door top at (14.06, 9.59, −29.61) south-west into the ditch, aiming to cross (12.0, ~6.3,
   −32.0): does the real game (a) fall forever into the void like the port, or (b) reset /
   kill / respawn Bond? If (b), the port needs the same reset (a game fix — the spot is
   reachable by a player, and the strips are x 11.1-13.4 / 15.6-17.5+, z −30.4..−35.6). If
   (a), the port is faithful and no code change is warranted — the route work already keeps
   the walk off them.
2. A second confirmation point on the same strips: from the moat band's east end (9.95, 5.84,
   −34.41) step EAST across x 10.75→11.5 at z −34.5 — the same void should open immediately.

## What's left

- The 00 Agent sister is still `#[ignore]`d per its own ledger (the rally approach crossing
  the door-8 gun line, the 87000-tick door stall, the exit-lawn #16 volley) — the follow-up
  task `real-health-00agent-02-03-3` owns it.
- The four-crate suite on a window with room (see §2's scoping note).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the session.
