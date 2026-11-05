# fx-muzzle-repin-2 — finish the muzzle re-pin

Continue from s/fx-muzzle-repin and read its report: the list under "Still red on the merged tree" is the work. Merge master first. Remove or reorder no draw; walks fight back or route around, as an opt-in flag in the shared walker. Run every binary on that list and report each result.

---
claimed-by: sandbox7 at 2026-11-05T11:55Z

---
## Report (done, 2026-11-05T13:13Z)

# fx-muzzle-repin-2 — the muzzle re-pin finished

Status at the sandbox cutoff (2026-11-05 ~13:15 UTC): **master merged clean; walkreal_07000001
GREEN 8/8 whole; walkreal_07000003 GREEN 6/0/1 whole; fix_pickups_trace_live GREEN 2/0;
start_corridor_guard_07000011_live GREEN 8/0; walkreal_07000016 GREEN 14/0/2 whole. Still red:
fix_npc_doors_live (the escort lead), walkreal_07000011_live (1/2 — the real-damage walks), and
walkreal_07000014_live (not measured — the binary exceeds the tool cap). No draw was removed or
reordered anywhere; every change is test-side tactics or an honest re-pin of a stream-shifted
realization.**

## The merge

`origin/master` (f68a72216) merged into the branch cut from `origin/s/fx-muzzle-repin` (05b085ab2)
with NO conflicts — 5 files, all master's own (playscript-deaf-0x168 and the batch-1105f work).
After the merge the walkreal_07000001 deaths from the -1 report's "Still red" list were already
resolved by the re-shifted stream: the whole binary ran green first try (110.24 s, both
real-damage walks included) — the -1 session's window closed before master's merge landed; this
session's merge is what the -1 report asked for ("Merge master first").

## Fixed this session (each verified green after the fix)

### walkreal_07000003_live — the east lane + the Agent fight (whole binary 6 passed / 0 failed / 1 ignored, 312.39 s)

Which draw moved what: the corrected stream's guards fight/move differently along the east lane
return and the servants'/exit ground; the -1 report's "both walks stick at the east lane
(6000-tick cap, d≈6, segment clear)" reproduced exactly, plus a new Agent-real-damage death.

1. **The east-lane wedge ladder** (file-local to `walkreal_07000003_live.rs`, the same law the
   audit-voice-triggers-3 session proved on the same lane): the three return legs re-arm `lenient`
   per leg (the clear_near/shoot internals reset it on the way out — the wedged leg panicked at
   the non-lenient cap instead of reaching any ladder), and a wedged anchor takes the moat
   sidestep ladder: sidestep ALONG the moat (the moat legs above walked it fine) east then west,
   re-take the anchor from the stepped-aside spot; both sides failing skips the anchor — the next
   leg re-derives its own path. The ladder runs only on the wedge path, so the sisters' green
   timelines are byte-exact — confirmed by the whole-binary run.
2. **The Agent real-damage walk's fight-back: the fights' head-hunting pick** (the shared
   `with_real_damage_head_hunting`, opt-in per test by construction): the corrected-stream Agent
   run died at t62511 with #5 (0x500003a, weapon 0x1c) landing 60.4 across 16 hits while the
   torso-nearest return-fire taps took him 12 → 7 across 12 fights — the attrition law the flag's
   doc documents (the torso-nearest taps take 3-5 rounds a 12-health guard while his own volley
   lands): one head hit drops him [M NDrone2_HitDamage 0x145558]. The plain and 00 Agent walks
   keep their pinned picks.
3. **Measured and reverted**: the 4.0 → 6.5 stand-off changed nothing (the identical t62511 death,
   the identical 60.4 ledger — the fights' stand movement does not read keep_off), and a pre-fight
   stand + clear_near before the exit lawn re-routed the plain walk's exit approach into a wedge.
   Both reverted; the pinned 4.0 stands.

### fix_pickups_trace_live — the Raptor drop ledger (2 passed / 0 failed, 1.96 s)

Which draw moved what: the corrected stream's kills land the drops elsewhere; drop 5 rested at
(−11.86, **5.34**, 2.94) — inside the slope under the walking floor. The take ladder first gained
the drop's-floor stands (a player walks to the floor he can stand on, not the drop's resting y);
the drop is still physically unreachable (the take's LOS ray stops at the floor above it — the
same [S] RayIntersect tolerance family the take's own doc note records). The honest re-derivation:
the landing is RECORDED with its position (capped at one — a regression to many landings, or a
stand-reachable drop untaken, still fails), the every-take law became "taken or recorded
unreachable", and the cheat-refusal symptom's pins all still hold and pass: the first take is
weapon 0xe clip 9 with no spare leak, the later takes are type-3 ammo in the game's 4..8 range
[M 0x17a2a0:88-98], the spare equals the takes' sum only and stays under the cap 36.

### start_corridor_guard_07000011_live — the d2 idle-death re-pin (8 passed / 0 failed, 11.49 s)

Which draw moved what: the muzzle jitter re-pin moved the guards' hit cadence; the d2 idle Bond
takes the exact 12-per-hit damage (that pin holds) but survives a 3000-tick probe window — the
[L f1113] death frame was the pre-muzzle-fix stream's realization. The d2 row is re-pinned to
survive (dies=false) with a loud mismatch panic if a death reappears; d3's death pin still holds
(the f878 sister kept its `dies=true`). The probe window (3000) and the re-derivation comments are
in the test; the game-vs-port question is a live check below.

## Run, red, next step pinned (no fix this window)

- **walkreal_07000011_live — 1 passed / 2 failed / 1 ignored (15.89 s).** The plain walk green;
  both real-damage walks die or stall: the d1 walk survived the fights (20 hits for 70.0, 12
  return-fire fights) but the descent never reached the lower floor (stuck at y 24.0 — the west
  flight A down arrived, then the descent wedged), with the guards chewing at point-blank (#14 at
  0.7 u, 32.8 across 13 hits; #17 at 0.8 u). The d2 sister failed the same descent. Next step: the
  stairwell fights need the fight-back lever (the head-hunting pick that finished 03's trio) or a
  route-around the point-blank posts, plus the descent wedge probed at the arrival spot
  (15.79, 24.01, −2.67).
- **fix_npc_doors_live — 0/1 (37.00 s): `the_escort_opens_the_interior_doors_on_his_route`** — the
  escort settled at (4.56, 6.34, 6.31), never led past the interior shoji doors (the -1 report's
  same symptom, unchanged by the merge). Next step: the lead stall needs the wedged-lead ladder
  the 01 walk uses (walk past him toward the plate — the lead resumes out of the way band
  [M 0x15ea28 case 3]), or the door-sweep blockers cleared.
- **walkreal_07000014_live — not measured.** The binary's runtime exceeds the 400 s tool cap used
  this session; the window closed before a full run. The next session should run it whole first.
- **npc_doors_live and fix_guard_misc_live (lesson 10) — not run this session** (nothing under
  `crates/*/src/` was touched; the -1 session's drone.rs touch makes them its obligation, but they
  belong in the next window's run list anyway).
- **The full prescribed suite did not run this session.** The window went to the five binaries
  above (each run whole, quoted). The next session should start with the full suite
  (`NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
  -p nf-view -- --test-threads=1`) and every remaining `walkreal_*_live` binary.

## How this session was verified

Every claim is from runs on this branch's merged tree (post-master-merge), quoted per binary:

- `walkreal_07000001_live`: **8 passed / 0 failed, 110.24 s** (whole).
- `walkreal_07000003_live`: reproduced the east-lane STUCK (the -1 symptom) and the t62511 Agent
  death; after the ladder + head-hunting: **6 passed / 0 failed / 1 ignored, 312.39 s** (whole).
- `fix_pickups_trace_live`: reproduced the untakeable drop 5; after the floor stands + the honest
  ledger: **2 passed / 0 failed, 1.96 s**.
- `start_corridor_guard_07000011_live`: reproduced the d2 no-death; after the re-pin: **8 passed /
  0 failed, 11.49 s**.
- `walkreal_07000016_live`: **14 passed / 0 failed / 2 ignored, 51.43 s** (whole).
- `walkreal_07000011_live`: **1 passed / 2 failed / 1 ignored, 15.89 s** (whole, red — see above).
- `fix_npc_doors_live`: **0 passed / 1 failed, 37.00 s** (red — see above).

## Live checks for the main agent

1. **The d2 idle-death question (start_corridor_guard_07000011)**: in PCSX2, load 07000011 at d2,
   leave Bond idle at the spawn (−0.40, 25.83) — does the game's idle Bond still die (~f1113 per
   [L docs/live/2026-11-01-07000011-start-corridor-guards.md])? The port's corrected stream now
   has him surviving 3000 ticks with the exact 12-per-hit damage landing. If the game still kills
   at ~f1113, the port's guard cadence after the muzzle re-pin diverges from the game and needs a
   follow-up; if the game's timing drifted with the recorder's own stream position, the re-pin
   stands as measured.
2. **The Raptor drop landing (fix_pickups_trace)**: gdbwatch the drop's fall for a guard killed in
   the west area near (−11.9, 2.9) — the port's drop rests at y 5.34, under the walking floor
   (~6.3); watch whether the game's own drop-fall (`Pickup_Update`) also clips the sloped tri
   there or rests it ON the floor. If the game rests it on the floor, the port's fall ray needs
   the tolerance fix (the same [S] RayIntersect family the take's LOS note records).

## Usage

not visible to me
