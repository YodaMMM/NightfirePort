# real-health-00agent-01-hall-6

Branch `s/real-health-00agent-01-hall-6`, cut from origin/master 385ebb66d with
origin/s/real-health-00agent-01-hall-5 merged in first (it was NOT on master; merge commit
e044dcd0b, clean). The task: take the hall-5 ignore message's named levers in order — the #7
volley-reaction race first, then the escort's west route, the slab ambush and the plate
AND-touch — each judged on its own real run, helpers gated to the level and the difficulty,
real input only, no top-ups, never lower damage or health.

## What's done

**THE #7 VOLLEY-RACE RACE IS WON.** The deep captain #7 (0x5000038, 24 hp) — the wall that
killed every hall-5 run at t3255 — now DIES AT TICK 3366 with Bond at FULL 55.0, killed by
the part-5 ×8 head record (24.0 → -8.4 in the final 12.0 round [L the run9 hit ledger; M
NDrone2_HitDamage 0x145558 head ×8]), and his first full volley never lands. The mouth guard
#6 (0x5000036) follows at 3525 (the detour walk's own return fire; the #7 fight's overflow
taps had ground him 12.0 → 2.4 stepwise [L the run10 NF_HEALTH_TRACE trace]). **Two of the
three AND inputs fire through the real fire path.**

The levers, in the ignore message's order, each on its own measured run ([L]
/tmp/h6_run1-13 on this sandbox, every one through the real input path from the level load,
no teleports):

1. **(a) the SMG pre-arm during the entry's quiet waits** — a new d3-gated
   `d3_switch_to_smg` (the deepest NON-pistol pool via the real weapon-select input
   [M Player_Weapon 0x1a24c8's pad select]) runs BEFORE the crouched door press: noise-free
   (no shot fired, the 91-tick reaction clock has not started) and door-closed quiet. Run 1:
   "pre-arming weapon 0x12" before the press, the fight taps then at the SMG's own refire —
   run17's in-window 41-tick draw is gone. The SMG's refire measured 6 ticks (the run4 hit
   ledger: a hit every 6 loop ticks) vs the pistol's 16.
2. **(c) the volley-race sort** — the d3 named sort now orders the 0x1c guards DEEPER first
   (the live positions at fight time), so the FIRST tap targets #7 (z 17.4) and not the
   mouth #6 (z 14.7). Run 1's log: "hall guard 7: lane stand picked" first. The race clock
   now starts on the volley owner.
3. **(b) the head-box aim law, measured run by run** — the decisive lever:
   - run4/5: the landings sit ON the aim point ±0.36 — while a part is exposed the aim rode
     the PART's own point (the lane heights never ran), all 20+ hits read body parts
     (4/50/53/54/35, 0.9-1.2 each), #7 ground to 2.0 — TWO HP SHORT.
   - run6: the dy-1.4 lane aim (the "muzzle drop" hypothesis) flew OVER everything — 60
     rounds, zero hits, #7 full 24.0. The drop hypothesis is dead: landings track the aim.
   - run7: the dy-0.6 aim bought ONE 12.0 part-5 head hit (24.0 → 10.8) and then #7
     sidestepped — 18 rounds found nothing.
   - run8: the dy-0.3 aim degraded to 11 hits for 12.6.
   - run9 (THE WINNER): for a NON-pistol held gun the lane aim OVERRIDES the exposed body
     part (dy 0.05 first — the torso center — so the taps keep firing when the part
     unexposes), the d3 gates tightened 0.05/0.06 → 0.035/0.04. The SMG's 6-tick refire
     grinds 20 hits (17×1.2 + 3×0.9) and the final round lands the 12.0 head record.
4. **the resupply wall (the run10 death at 10270)** — with both shotgunners dead the
   on-trail rule walked Bond DEEP into the hall: the on-trail stock took (7.1,5.3,14.3) then
   (6.3,16.8) and a west 0x1c volley killed him at (6.76,14.88) — the west lanes own the hall
   interior (hall-4's lever wording). Hall-4's detour-only lever is now real: the shared
   resupply grew the opt-in `no_fetch` (a spot ban) and `no_fetch_z_above` (the zone form —
   the (7.1,14.3) box SPAWNS after entry, so the entry-time spot list missed it; the run12
   ledger caught that), and the d3 walk bans everything across the z 11.2 door plane plus
   each dead guard's own drop spot. Run 13: no interior attempt. Both filters default
   empty/None — every other walk byte-identical.

The walk then runs to t12212 (from master's t6054 death spot and hall-5's t3255) and dies on
the last named guard.

## The remaining wall (the refreshed ledger, in the ignore message)

**g11** (the third named guard, 12 hp pistol): with the wedging west cells dropped from the
lane-stand pick (hall-3's own wedge ledger: the x 4-5 band stops 1.6-1.7 u short — the run12
STUCK at (5,10.6) d 1.27 confirmed it on this tree), NO reachable stand reads a clear lane to
him; the fight idles its 2400 ticks (run13: dead false, g11 hp 12, bond 26.2, at 12212), the
third AND channel never fires and the walk's asserts fail. A non-fatal detour return-leg
wedge also showed (back-to-hold nav 4, d 1.77, segment None). NEXT LEVERS, concrete:
(i) g11's kill — hall-4's LAST-GUARD PRESS shape wired into the named loop's tail for the
last across-alive named guard: the shallow press to (6.5,13.2) draws the rusher's charge, ONE
wound-aborted shoot attempt finishes him at close range (his parts expose inside the hall —
the hall-4 run3 pass killed him from z 19.4), the crouched retreat home between legs;
(ii) the detour return leg's wedge — the hold-line return at arrive 1.2, a wedged leg falls
back to the deep line (6.5,7.6). Then the escort's west route, the slab ambush and the plate
AND-touch (hall-4's post-AND machinery is in the tree, untouched by this task).

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **7 passed, 0 failed,
  1 ignored** [L 70.5 s] — the ignore is the d3 00 Agent ledger, rewritten to the hall-6
  state above. The d1 real-damage and d2 Agent sisters pass unchanged (every change is
  difficulty-gated: the new helper calls, the sort, the aim override, the gates and both
  no_fetch forms are d3-only; the shared `no_fetch`/`no_fetch_z_above` fields default
  empty/None and the code checks them).
- `interactables_07000001_live`: **10 passed, 0 failed** [L 29.6 s].
- 13 instrumented runs ([L] /tmp/h6_run1-13 on this sandbox): run1 the pre-arm + the race
  sort first firing; run2 NF_DMG_TRACE + the drone watch; run3 NF_HEALTH_TRACE; run4/5 the
  per-hit landing ledger (the instrumentation prints every guard health change with the part
  and the point); run6 the dy-1.4 disproof; run7 the dy-0.6 head hit + the sidestep wall;
  run8 the dy-0.3 degradation; run9 THE KILL (#7 dead at 3366, bond 55.0); run10 the health
  trace (d#6/d#7 death ticks, the interior-fetch death at 10270); run11 the no_fetch spot
  ban (compile: E0609 — the field lives on the other Walker); run12 the zone-form discovery
  (the (7.1,14.3) box fetched despite the spot ban) + the g11 wedge at (5,10.6); run13 the
  final shape (#7/#6 dead, the interior fetches banned, g11 the remaining wall).
- The changed files: `crates/nf-view/tests/walkreal_07000001_live.rs` (the d3 levers, the
  hit instrumentation, the ledger) and `crates/nf-view/tests/common/walker.rs` (the two
  opt-in resupply-ban fields + the filters; both default off; the field added on its own
  line in the one `Walker` literal). Per the merge lessons I grepped the tests that use the
  touched code: the walker.rs fields are only read in `resupply` (all levels' walks call it
  — the whole walkreal battery above is the proof the default is byte-identical; the other
  levels' walkreal binaries were NOT all run this session — the shared-helper rule's full
  battery is the one gap, covered only for 07000001).
- **The full 4-crate suite did not run**: this 9.9 G sandbox hit ENOSPC twice during the
  first build (the nf-view test binary forest is ~5.5 G; I deleted all but this level's two
  binaries to fit — the other levels' binaries would relink for the suite and hit the same
  wall), and the suite's own history on this disk size says it cannot fit (the hall-4
  report's note). The two whole level batteries above are the coverage; re-run the suite on
  a bigger disk before merging.

## What's left

- The g11 lever (i)/(ii) above, then the escort's west route → the slab ambush → the plate
  AND-touch → the exit; the ignore message carries the refreshed ledger.
- The full suite on a disk that fits it; every other level's walkreal binary re-run (the
  shared-helper rule) once the binaries can coexist on disk.

## Live checks for the main agent (PCSX2, fresh boot)

1. The SMG's torso-rate kill: from the meet-room step (≈7.0, y-step, 9.5) with the meet SMG
   held, confirm sustained hip fire at the hall's deep captain lands torso hits at the
   measured ~6-tick refire and that a part-5 head record pays 12.0 — the port's kill needed
   20 hits, 17 torso + the final head.
2. The on-trail resupply rule: with the hall door open and the interior boxes on the walked
   path, confirm the real player does NOT backtrack into the hall interior for the
   (7.1,14.3)/(6.3,16.8) boxes while the west corridor is under fire — the port's ban is
   harness policy; the disc's own punish (the west volley) is the evidence.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~09:52 UTC; this report written ~09:06 UTC), `git` hashes, test outputs and process
exit codes. No input/output token counters, no credit balance, no dollar cost.

## ntfy failed

All three fallbacks failed; the exact outputs:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "real-health-00agent-01-hall-6: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'…', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>` (exit 1).
3. `wget -q -O- --post-data "real-health-00agent-01-hall-6: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → killed at the tool timeout (30 s) with no output; re-run under `timeout 25` → exit 124
   (timed out), still no output.
