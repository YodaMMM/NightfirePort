# Bring back the three ignored walks

Branch from `origin/master`, push to `s/civ-unignore`. Three walks are `#[ignore]`d because the scripted bot does not survive the now-real fights: walkreal_0700000a_live _00agent, walkreal_07000013_live real_damage, walkreal_07000006_live _agent. `origin/s/civ-repin-finish` and its report hold what was tried. Test-side only. Remove each ignore as its walk goes green. Do not loosen a pin tagged [L]. Run each binary whole and quote the result.

---
claimed-by: sandbox4 at 2026-11-05T23:21Z

---
## Report (done, 2026-11-06T00:17Z)


---
## Report from the worker (done, 2026-11-06T00:18Z)

# civ-unignore — report

Branch `s/civ-unignore` (from `origin/master` 576909ed6, the civilians-raise-the-alarm
merge). 2026-11-05/06, written ~00:20 UTC. Task: bring back three `#[ignore]`d
real-health walks, test-side only, removing each ignore as its walk goes green.

## Result in one line

**1 of 3 green and unignored (0700000a 00agent); 07000013 measured to a structural
answer with one shipped opt-in fix but still red (ignore kept); 07000006 not started
(this session's window went to the 13 ledgers); no shared file outside the named test
files + one opt-in `tower2` flag; no [L] pin touched.**

## walkreal_0700000a_live 00agent — GREEN, ignore removed

`walk_office_floor_qworm_keyboard_hatch_real_damage_00agent` (d3). The walk died on the
doorway crossing at (−10.45,−18.65): 25 × −4.0 from "the world" — which the trace shows
is wall gun **#639** (a GT turret's rounds carry the owner usize::MAX mark [M drone.rs
17828's GT Fire arm], so they print as the world).

Measured with the game's own track rule — a probe walking the level after the keypad arm
and sampling `world.line_of_sight(turret.pos, p, LOS_TURRET_TRACK)` [M drone.rs:17703]
plus the `dot(fire_dir, dir) ≥ 0.01` front cone [M gt.rs `GTurret::track` 0x17f9c8]:

- **#639's fire_dir is (0,−1,0)** (every 0a wall gun's is): the cone passes every floor
  point within 50 u, so the cone is no protection on foot.
- **Every pocket cell x −17..−10.5, z −20..−15 reads LOS-clear** — the merge
  576909ed6's world/LOS state left the whole pocket exposed, so the master walk's "out
  of the turret's line" retreat to (−14.5,−15.0) bought nothing: the measured master
  ledger shows the retreat's own seg 0 eating 11 rounds (69 ticks), and the deadly
  window is #639's COMBAT state (script-timed after its first track) until the LOS
  blocks east of x ≈ −10.0 (the probe; the d2 ledger's hits stop exactly there).

The fix (test-side, the walk file only): **the retreat is removed; the 4 digits type AT
the wedge** — the armed keypad is a MENU state, position-independent [M
C_KEYPAD_Handler 0x216810 + 0x1ae3d8 case 4] — and the doorway crossing runs straight
from the wedge, so the whole post-arm tail ends inside #639's first ~20 COMBAT rounds.

**Whole-binary result (quoted, `--test-threads=1`, final tree):**
`walkreal_0700000a_live — test result: ok. 13 passed; 0 failed; 2 ignored` (50.13 s).
The 00agent ledger: 21 rounds for 84.0, first hit t12409 at (−12.3,−17.2), the last
t12549 at (−9.8,−18.6), end health 16.0. The d1/d2 sisters' ledgers shrink the same way
(the d2 run's 30-hit razor pass becomes the same ~21-hit tail). The 2 remaining ignores
are the pre-existing doorway-crossing research tests, not walks.

## walkreal_07000013_live real_damage — still red, ignore kept, one fix shipped

`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (d3). Master baseline:
death t1810, the 12 walk delivers **62 hp + 50 armour** (−14 the bend fight #3, −24
before the corridor fight), the 13 lobby swarm consumes the 62 exactly.

**Answered civ-repin-finish's first live check** (what wrote 86.0 → 62.0 between ticks
2531 and 2764): `NF_DMG_TRACE` shows it is guard **#10's opening fire — ONE tick of 8 ×
−3.0 shotgun pellets (weapon 0x1c) at d 3.5** (t2723, Bond at (16.8,−12.0) mid "the
south corridor" leg), not a grenade and not mover damage; his burst's own fight then
killed him at t2764.

Shipped fix: `tower2::PRE_CLEAR_SOUTH_CORRIDOR` (pub AtomicBool, default false, checked
at exactly one labeled leg of `walk_12_exit` — the opt-in pattern rule 7 requires). The
d3 walk sets it; the stand runs only if the game's own `exposed_part` probe reads a
reachable box from "the south-east bend 2"'s leg end (d 4.4 to #10) and kills him
through the ordinary fight machinery before his sight opens. Measured: **carry into 13
went 62 → 86 hp + 50 armour; the pre-clear stand itself took zero rounds** ("killed
true" at the bend-2 end; the earlier corridor-east stand point (d 1.8) took −14 in the
stand and was moved to bend-2). Tower2's other callers run byte-identical (flag off).

Still red: with the carry at 86 the swarm dealt **86.0** in 24 hits (death t1666). Three
carries measured this session: **62 → 62 (t1810), 72 → 72 (t1745, the corridor-east
pre-clear), 86 → 86 (t1666, the bend-2 pre-clear)** — the fight-loop attrition is
uncapped, not carry-limited: the return-fire loop stands in the open lobby lane and
trades until Bond dies, whatever health he carries. Carried-lever arithmetic cannot
close that; the missing machinery is fight POSITION (cover), which is src/ machinery,
outside this task's test-side scope. The repin-finish (a)+(b) levers were re-measured on
this master (shooters_window (20,6) + back_off 5.0) and made it WORSE (death t1509) —
not shipped.

**Whole-binary result (final tree):** `walkreal_07000013_live — test result: ok. 2
passed; 0 failed; 1 ignored` (24.88 s) — the d2 agent sister and the plain chain pass;
the d3 real_damage test keeps its `#[ignore]` with the new evidence in its doc comment.

## walkreal_07000006_live agent — not started

The predecessor (docs/reports/civ-repin-finish.md) measured five configurations to a
hard blocker (Bond's muzzle rays to the strip posts read blocked from every west anchor
at 21-28 u while the posts' rounds reach from d 9.5). This session's window went to the
13 ledgers; the branch carries no 06 change, its ignore is untouched, and the parked
config + live checks below are carried over unchanged.

## Verification (every binary that uses code I touched, run whole)

- `walkreal_0700000a_live` — **13 passed / 0 failed / 2 ignored** (50.13 s).
- `walkreal_07000013_live` — **2 passed / 0 failed / 1 ignored** (24.88 s).
- `campaign_flow_live` — **5 passed / 0 failed** (298.56 s).
- `guards13_wake_live` — **1 passed / 0 failed** (9.88 s).
- `guards13_warm_probe` — **1 passed / 0 failed** (9.86 s).
- `fall13_probe` — **0 passed / 0 failed / 1 ignored** (the probe is ignored by default).
- `walkreal_07000011_live` — **3 passed / 0 failed / 1 ignored** (38.33 s).
All with `--test-threads=1`, NF_ISO set, this branch's final tree. The full suite did
not run: only the three named walk files + the opt-in tower2 flag changed, and every
binary that compiles tower2 or the touched tests is listed above (the `walkreal_07000006`
binary compiles neither and its walk tests are difficulty-independent of this change).

Determinism untouched: no RNG draws added (the pre-clear runs the fight machinery's own
path), no wall-clock, no position writes, no teleports; every step is the real input
path.

## What is left

1. **07000013**: the fight-position (cover) answer — where the return-fire stand should
   TAKE ground (the lobby's sight-shadow spots) instead of standing where the 10-tick
   check fires. Src-side or a bigger walker lever than this task allowed. The measured
   ledgers (62/72/86 all consumed; deaths t1810/t1745/t1666; the (a)+(b) re-measure
   t1509) are the baseline to beat; the pre-clear flag ships and works (carry 86, stand
   clean).
2. **07000006**: the parked config (guardroom clears to d2) is measured red on the
   predecessor's branch; the ray-blocker question below is the gate. With the answer,
   either a stand anchor that opens the rays or the proof the pocket must be avoided by
   route.

## Live checks for the main agent

- **07000013 spawn lobby (gdbwatch)**: during the d3 lobby fights, record each fight's
  start position vs the shooter's sight cone — is there a spot within 3 u of the lane
  the lobby's live guards cannot see (the sight-shadow map)? That spot is the cover
  anchor the fight loop needs; the walk can be walked THROUGH it before the fights open.
- **07000006 (fieldprobe, carried over from civ-repin-06, decisive)**: the
  exposed_part_gun_rays game twin (the 0x1d8f20-family ray set) from (a) the strip
  anchor (6.0, 42.0, −34.0), (b) the corner (14.0, 41.9, −36.7) and (c) the east floor
  (25.3, 42.6, −40.6) to the strip posts #24 (28.1, 42.0, −36.0) and #5 (27.8, 42.0,
  −35.7) — which of the three anchors has ANY clear part ray, and at what distance the
  gate opens. Expected: (c) opens (the reactive fight ran at d 9.7 between (b) and (c)),
  (a) does not (four censuses read rays0 at 21-28 u).

## Usage

not visible to me — this environment shows no token, credit or context-window figures
for the session, so there are no numbers to copy.
