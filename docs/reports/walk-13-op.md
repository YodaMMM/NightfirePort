# s/walk-13-op: the 13 warm-chain real-damage walk (00 Agent) — the lobby-budget lever taken, ledger refreshed, still red

Branch from origin/master (2bd059afe). Task: read the ignore ledger on
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (walkreal_07000013_live), skip
the measured-dead levers, take the next untried lever from the ignore message (s/hard-walks-b:
"the 13 lobby budget from the re-anchor end on"), judge each lever on its own real run.

## What was done

**Reproduced the ledger byte-identically first** [L this tree's run]: died t5119 at
(-4.92,-368.39,39.07) with the re-anchor pool fill ending at 13.5 health, exactly the
s/hard-walks-b ledger. The 13-side damage breakdown (NF_DMG_TRACE): the mezzanine pistol guard
#5 (hp 12, unkillable — see below) dealt 37.5 in 17 hits across the walk; the balcony rifle #4
finished the walk (10.0 in 6); the pool fill's three passes bled 34.5 (pass 1 -13.0, pass 2
-14.0 for +7 held, pass 3 -7.0 for NOTHING — held 21 -> 21, the fetched gun went to inventory
and the pass still returned got=true via resupply's arm-switch fallback).

**The lever (the ledger's own, taken): the 13 lobby budget from the re-anchor end on.**
Two pieces, both judged on the lever's own real runs:

1. **The pool fill runs BEFORE the mezzanine pre-emptive stands, and one no-gain pass ENDS
   it.** The stands' own law ("top the magazine before taking a sniper's post", the lever-4
   ledger) is now armed from the pool's depth, and the exit fill's own ammo-rise law applies at
   the pass level. Measured on the lever's run: pass 1 fetched the guard 0x19 drop whose gun
   LANDED IN HAND (held 15 -> clip 8 spare 7) for ZERO health — the s/hard-walks-b tree's pass 1
   cost -13.0 — and the s/hard-walks-b fill's -21.0 of later waste (passes 2-3) never runs.
2. **The re-anchor sweep's floor stands run the file's own measured CHARGER LAW** (`shoot_loose`,
   the retune-nb-13-4 door-drain lever, d3-only `loose` param on the file-local
   `clear_the_visible`; every other call site passes false). The lever-1 run (fill-first
   without the loose law) died t4000 INSIDE the sweep's still-gate stand on the wave Spas12 #7
   (hp 12, d 7.0): the aim phase held fire while his 8-pellet TooClose burst landed -24.0 in
   two frames [L the lever-1 log]. With the loose law the sweep killed #7 for 0.0 dealt [L the
   lever-2/3 logs].

**Probed and reverted: the fights' unseen-cover fallback (`UNSEEN_FIGHTS`).** The lever-2/3
runs recorded 0 return-fire fights while two unexposed shooters chewed the walk. The opt-in was
set d3-only (a pub setter in damage_log.rs, reset at the walk top per the per-thread law) and
measured a BYTE-IDENTICAL death — no-op — and was reverted; damage_log.rs is back to pristine
(git diff: only the test file changed). The reason it cannot fire there: the deaths come inside
the RAW stands (the sweeps, the far stands), which never run the fights' pick at all.

## What still kills (the refreshed ledger, now in the ignore message)

The kills RE-FEED the lobby spawner (start ch 0x29 [D §4]): after the sweep's #7 kill, the
re-fed wave guard #10 (skin 0x5000008, 0x19, hp 12) stands d 4.3 at the sweep's own spot and
the sweep's SECOND stand loses the point-blank trade (-24.5 in 7 hits from #10 + -23.0 in 12
from #5) — death t4784 at (-0.55,-368.39,30.89) inside the stand [L the lever-2/3 logs,
byte-identical]. #5 stays unkillable from the lobby floor: the mezzanine balustrade face
x -16.29 stops EVERY round (the 0x1c shotgun's pellets [L the MEZZ_DRY_END ledger] AND the
0x19's single rods — hp 12 intact all run [L the walk-13-op runs]) while his box reads exposed
(the eye/gun-ray pick returns Some). The lever-1 pre-emptive stand on him cost -9.0 for nothing.

**Two follow-up levers probed on their own real runs (after the ledger above was written):**

3. **TAKEN — the sweep is capped at its ONE #7 kill (stop-on-success, d3-only).** The #10
   point-blank trade is GONE from every later run (#10 dealt 10.5 from d 13.1 mid-leg on the
   walk-on run, never the sweep's d 4.3 trade). Its own run still died t4337 mid-leg: the
   re-dealt RNG fed the balcony rifle #4's fire early (21.0 in 7) plus #5 (26.5 in 13), 1 fight.
4. **MEASURED DEAD, reverted — the walk-on line (return_fire OFF) extended one leg earlier to
   the west-mid leg.** Death t4242, the post trio ALL firing at the MOVER (#5 23.0 in 12,
   #4 14.0 in 6, #10 10.5 in 3, 0 fights) — the sight law fires at movers too once the guards
   are engaged, and the answer block never got its health budget [L both logs].

**Next lever (in the ignore message): run the west-mid ANSWER BLOCK at the sweep's kill spot**
(the re-anchor end, the pre-emptive's own dy>2 pick law — the mezzanine far stands, the
0.02/0.028 band that converged on #4 at 21.4 u killed-true [L the nb-13-2 ledger]) one leg
earlier than the west-mid end, because all five walk-13-op runs measured the leg itself as the
death ground; the floor answers (shoot_loose, the charger law) follow at the same spot. #5's
face (x -16.29) still blocks every round from the lobby floor — the live check stands.

## How it was verified

- The ignored d3 walk alone, four runs (baseline repro + three lever runs), NF_DMG_TRACE /
  NF_RF_TRACE captures (logs in /tmp, not committed).
- The WHOLE walkreal_07000013_live binary on the final tree (twice: after the first ledger
  push and after the follow-up levers): **4 passed, 0 failed, 1 ignored** each time
  (the plain walk, the agent sister, the operative sister and the route-flag thread test all
  green — the sisters stay byte-identical: every lever lives inside the d3 gates, and the d2/d1
  code paths are untouched).
- interactables_07000013_live (final tree): **10 passed, 0 failed**.
- The ignored d3 walk alone on the final tree: died t4337 (the stop-on-success run's shape, the
  refreshed ledger).
- No sim, draw, RNG, damage, health or guard-code change; no top-ups; real input only; every
  position/stand rides the walk's own machinery.

## What's left

- The walk is still red (the ledger above). The next lever is named in the ignore message.
- The full suite (tools/test/suite.py) did NOT fit this sandbox: background jobs are killed
  between tool calls here (the first nohup suite start died at 20:27 UTC with 0 output), the
  box has 2 cores, and the full build of every test binary alone outlasts the window after the
  lever work (two 9.5-minute chunk_suite invocations left the build unfinished — the cargo
  cache keeps the progress; a later session can resume `python3 tools/test/chunk_suite.py
  <state> --budget <s>` until ALL DONE). The changed code's own binaries ran green whole (the
  counts above); `git diff` shows damage_log.rs pristine and only the test file changed, so no
  other binary's behaviour can differ from master.

## Live checks for the main agent

1. The mezzanine balustrade face x -16.29: from the re-anchor spot (3.29,-368.39,31.77), does a
   FRESH-BOOT player round (any weapon, aim held) at the mezzanine pistol guard #5
   (-17.7,-362.4,45.9) land on his box, or stop on the face? Expected per the port: stopped
   (the port's Collide trace). If a real player CAN kill him from the lobby floor, the port's
   bullet trace at that face is wrong — that would be the root fix, not a route lever.
2. The re-fed wave guard #10 at d 4.3 from the re-anchor spot: what does a fresh-boot player's
   point-blank tap do (the port's stand lost the trade at hp 12; a player should win it).

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed in this
environment; nothing to copy).

## ntfy

curl timed out after 20 s (`curl: (28) Connection timed out after 20002 milliseconds`), the
python urllib fallback timed out at 25 s; the wget fallback delivered (exit 0).
