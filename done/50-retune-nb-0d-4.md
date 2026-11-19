## Your task: `retune-nb-0d-4`, 0d arrival-to-C5-door 00 Agent walk green again (round 4)

Branch from current origin/master; if origin/s/retune-nb-0d-3 is not on master, merge it in first. Read docs/reports/retune-nb-0d-3.md there and the ignore message on walk_arrival_to_the_c5_door_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-0d-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-19T03:51Z

---
## Report (done, 2026-11-19T05:06Z)



---

# Worker report (docs/reports/retune-nb-0d-4.md @ s/retune-nb-0d-4 8cbe60768)

# retune-nb-0d-4 — the 0d arrival-to-C5-door 00 Agent walk, round 4

Branch `s/retune-nb-0d-4` cut from `origin/master` (73822d3a0) with `origin/s/retune-nb-0d-3`
merged in first (it is not on master). Task: take the next lever from the retune-nb-0d-3 ignore
message — the round-4 list (a) the corner-chip shave, (b) the #22 46.0 re-roll, (c) the 18-tap
zero-chip ride hunt — helpers gated to level 0d + 00 Agent, each lever judged on its own real
run, real input only, no top-ups, no damage/health writes, no guard-code changes.

## What's done

1. **The baseline reproduced byte for byte** (the committed 0d-2/0d-3 shift-30 defaults): the
   ride chips t18877 (g3 1.8) / t19384 (g30 1.8), 25.8 → 22.2 on the girder, the container
   landing 18.6 (fall timer 63), the exit 16.8, death t19772 at (141.22, 4.88, -296.08). The
   build matches the committed timeline; 17 fresh real runs this session, ~30 s each.
2. **Lever (c), the dice hunt — DEAD IN THIS NEIGHBOURHOOD** (twelve fresh rolls): `NF_0D_TAP_CADENCE`
   {7, 9, 11, 13, 14, 16, 18, 20} at the committed shift 30 ALL died ON THE RIDE to g18's
   22.2-24.0 0x12 draw (t19194-19292) — the default cadence 15 (19 taps) is the roll where g18
   misses, and every neighbouring tap schedule re-rolls it onto the rider; the shift-60 family
   (cadence 14/16) died the same way; `NF_0D_TAP_ZONE` 20/30 SATURATE to the default roll exactly
   (the same chip ticks, the same death t19772). The old branch's 18-tap zero-chip config is not
   reachable here.
3. **Lever (b), the #22 re-roll — MEASURED DEAD with a new law.** The strip-mouth head act (the
   round-1 box [209.0, -289.3] tol [7.0, 3.5], d3-gated env `NF_0D_DECK_KILL`, committed default
   0) landed each run but the box picks the NEAREST PRESENT guard — g11 (0x5000093, the bay
   captain), not #22 — and the re-deal is NOT stream-keyed: `NF_0D_DECK_EXTRA` 1/2/3 deliberate
   pre-shot draws produced the IDENTICAL deal tick for tick (#20 46.0 t2022, #36 48.0 t2060, #32
   6.0 t2599/2600, death at the ladder base t2599/2600). The 0x1e snipers' fire is
   sight-cadence-keyed and LANDS at d3 Accuracy row 2 regardless of the draw count — "each act
   variant is its own roll" is measured FALSE for this act (the stream only rolls the damage
   VALUE 46/48; the hit/miss saturates at row 2).
4. **THE ARMOUR TAKE — the round-5 breakthrough.** The kind-3 pickup EXISTS at 00 Agent:
   pickup 1, kind 3, weapon 0x6, rounds 50, at (205.41, 0.11, -205.31), bounds radius 0.81,
   gone false / hidden false / state 1 [L the probe run] — the placed pickups spawn with NO
   difficulty filter [D scene.rs `placed_pickups`: every flags&0xff==0xf0 placement]. The TIGHT
   approach (`walk_nav_to` arrive 0.2 + a 40-tick stand, env `NF_0D_ARMOUR_PROBE`, committed
   default 0) TAKES it: armour 2.0 → 50.0 [L] — the round-1 refusal was the LOOSE arrival
   (arrive 0.6 stopped 0.57 dxz / 1.03 dy off; the touch is the radius+1.1 sphere from the
   player's position + the LOS mask-0x721 ray from his world-sphere centre [M Pickup_Update
   0x18e148:203-214, two different origins]). This ANSWERS the kind-3 pickup live check carried
   since retune-nb-0d. The probe roll still died (t4023 at the ladder base, before the climb):
   the 1388-tick walk-over dwell sits in the snipers' sight (#32 100.0 in 4 hits; the fresh 50
   absorbed ~1-2 rounds then spent — the probe ledger shows both -0.00 armour-eaten entries and
   a -46.00 health entry, so the exact absorb order needs the [M] pin of Player_HandlePain
   0x1902c8:84-104).
5. **Lever (a) (the corner shave) went untried** — the budget went to the dice hunt, the deck
   act and the armour probe. It stays in the ignore message.

## How this was verified

- Seventeen real runs of the d3 walk this session (`--ignored --nocapture`, each its own roll,
  NF_0D_AUDIT on for the bullet-spawn evidence): the baseline, the eight cadence rolls, the two
  shift-60 rolls, the two zone rolls, the three deck-act rolls (extra 1/2/3) and the armour
  probe roll.
- The whole `walkreal_0700000d_live` binary on the final tree: **3 passed, 0 failed, 1 ignored**
  (102.0 s) — the plain master replay, the d1 real-damage walk and the d2 Agent sister all pass
  (the new machinery is env-gated, committed defaults 0 = the master timelines byte-identical);
  the d3 walk stays ignored with the refreshed ledger.
- `interactables_0700000d_live`: **11 passed, 0 failed** (27.6 s).
- The fast runner (`tools/test/suite.py`): started in the background three times (once early,
  twice on the final tree, the last with setsid); the sandbox killed it between tool calls every
  time — `suite.out` stayed empty, `ps` found no suite process right after each check (the same
  background-jobs-die signature rounds 1-3 recorded). The per-binary foreground runs above are
  the primary evidence, as in rounds 1-3. Crates the tests touch: only `crates/nf-view` (test
  files); `nf-game`/`nf-formats`/`nf-audio` sources are untouched by this branch.

## What's left

- The walk itself: the round-5 levers are in the ignore message — (A) THE ARMOUR WALK-OVER
  RESHAPE (the take is proven; the cost is the walk-over dwell itself — a shorter/faster path,
  or folding the take into an existing north-bound leg, or holding the armour until the climb;
  the absorb law first needs the [M] read of Player_HandlePain 0x1902c8:84-104. With the armour
  alive at the pre-ride phases, #22's 46.0 every-roll round eats armour and the floor sweep's
  budget lands ~+40); (B) lever (a), the corner-chip shave; (C) the exit-fall crouch press after
  the exit's last multi-landing step (one press — the burst's visibility 0.75× + the 0.45 sight
  drop may cut the landing burst's 6 chips).
- The d3 gate is still `#[ignore]`d with the refreshed ledger (rounds 1-4 measured).

## Live checks for the main agent

- The kind-3 pickup live check from rounds 1-3 is ANSWERED (the probe): the pickup exists at d3
  and takes with the tight arrival — no fieldprobe needed. Carried live checks from rounds 1-2
  (the arming event's address on g0; the 18-tap roll's config in the main agent's session notes)
  stand as written in `docs/reports/retune-nb-0d-3.md`.
- None new this session: the armour absorb law is a decompile read (Player_HandlePain
  0x1902c8:84-104), not a live check.

## Usage

not visible to me — this session's environment shows no token, context-window or cost figures,
so there are no numbers to copy.

---
## Report from the worker (done, 2026-11-19T05:18Z)

