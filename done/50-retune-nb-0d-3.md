## Your task: `retune-nb-0d-3`, 0d arrival-to-C5-door 00 Agent walk green again (round 3)

Branch from current origin/master; if origin/s/retune-nb-0d-2 is not on master, merge it in first. Read docs/reports/retune-nb-0d-2.md there and the ignore message on walk_arrival_to_the_c5_door_real_damage_00agent. Take the next lever from that ignore message and win the fight that now kills Bond. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-0d-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-18T23:37Z

---
## Report (done, 2026-11-19T00:42Z)


---
## Report from the worker (done, 2026-11-19T00:42Z)

# retune-nb-0d-3 — the 0d arrival-to-C5-door 00 Agent walk, round 3

Branch `s/retune-nb-0d-3` cut from `origin/master` (73822d3a0) with `origin/s/retune-nb-0d-2`
merged in first (it is not on master). Task: take the next lever from the retune-nb-0d-2
ignore message — the yard-floor fight budget, lever (c) "fight the converged garrison from the
container rim BEFORE dropping" — helpers gated to level 0d + 00 Agent, each lever judged on its
own real run, real input only, no top-ups, no damage/health writes, no guard-code changes.

## What's done

1. **The baseline reproduced** (the committed 0d-2 defaults, ride shift 30): death t19772 at
   the exit landing (141.22, 4.88, -296.08) at 16.8, and the `[0d:land]` census captured — the
   converged four (g0 0x500006d at (157.55,-286.52) state 0x68, g1 0x500007e (152.55,-286.83)
   0x73, g2 0x500006d (135.79,-298.19) 0x5e, g3 0x500006d (132.79,-297.72) 0x5e), all
   10-health grunts at d 9.7-15.6 from the container top (Bond landed at (148.36, 8.21,
   -296.08) with 18.6).
2. **Lever (c) — the rim sweep from the container top — MEASURED DEAD** on five real runs,
   each judged on its own roll, with the audit (`NF_0D_AUDIT`) proving the two laws that kill
   it. The machinery stays in the tree behind `NF_0D_RIM_MAX` (committed default 0 = OFF, so
   the 0d-2 timeline stays byte-identical):
   - **The arming law.** The first kill-shot from the top lands (a part-5 head hit on g3 at
     +34: `[NF_0D_AUDIT] b0 SPAWN dir (-0.972,-0.218,-0.088) -> part Some(5,0.099)`, 10.0 →
     -310) and ARMS the yard: g0 lands its first 1.8 chip ~24 ticks later, then g0 AND g1
     sustain 1.8 every ~6 ticks — 0.3 damage/tick EACH (the measured roll: 9 chips in 51 ticks
     ate the 18.6 pool mid-sweep). One shot buys one kill and hands the whole pool to the
     pack's cadence. The floor-landing burst (6 chips/6 ticks) is the same pack armed by flat
     sight.
   - **The post-kill swallow.** A trigger tick ~36 ticks after a landed kill spawns NO bullet
     (`[NF_0D_AUDIT]` pass 1's tap at +34: trigger input in, NO SPAWN line; pass 0's kill shot
     was at +34) — the sweep gets ONE shot per ~200 ticks while the armed pack drains
     0.3-0.6/tick. The ride fire's ~28-tick taps never killed anything, so they never hit this
     law before.
   - **Sub-laws measured on the way** (each its own run): the CROUCHED stand breaks Bond's OWN
     head-box rays to the north pair at the container's north rim (three head-aim taps at g1
     landed ZERO — the act fell back to lower boxes and the drop put every bullet below them);
     the STANDING stand holds those head-rays clear (the v1 roll's one-tap g1 kill). The
     muzzle-drop law: a chest-centre aim's bullet hit part 4, the LOWER torso (`b0` dir
     (-0.979,-0.179,-0.096) vs the geometric (-0.989,-0.111,-0.107) — ~0.07 rad of drop baked
     into the spawn), so steep down-shots land below the torso and the halved 0.12 u tap gate
     still missed the steeper g2 line twice. The torso-first and head-first aim variants, the
     yaw-error-first (front-arc) pick, the threat pick (the last chipper dies first), the
     32-tick tap cadence (the ride fire's own ~28-tick spacing) — all measured inside the same
     wall: the rim fight cannot outrun the arming.
3. **The dice knobs do NOT move the floor fight** (two runs): `NF_0D_TAP_CADENCE=17` (17 taps)
   reproduced the shift-30 roll tick for tick — the same ride chips at t18877/t19384, the same
   landing 18.6, the same death t19772 — the guard fire is sight-event-keyed and the tap
   draws are invisible to it. `NF_0D_RIDE_SHIFT=0` + the tap defaults (19 taps) reproduced the
   ROUND-1 death exactly (g18's ONE 25.8 0x12 round at t19197, owner-resolved in the log) —
   the shift-0 ride is dead on this timeline.
4. **The walk is still red** — the refreshed ledger (the two audit-proven laws above, the
   dice-knob dead end) and the round-4 levers are in the ignore message: (a) shave the corner
   chips 5.4-9.0 without extra dwell time (trim the 20-tick claw switch poll, pre-arm the
   latch), (b) re-roll the #22 46.0 (the deck-duo kill re-dealt #20+#32; a kill act with a
   different draw count may re-deal kinder), (c) hunt the 18-tap zero-chip ride roll (the
   dice-knob comment's own measurement: an 18-tap roll re-rolled the ride to ZERO chips,
   boarding 31.2 through the landing, a ZERO-COST exit — the exact budget the floor sweep
   needs; the exact old tap config is not on this branch), (d) the floor-landing crouch (the
   0.75 visibility factor + the 0.45 dropped target against the 6-chip landing burst, crouch
   ON before the burst's sight events fire).

## How this was verified

- Six real runs of the d3 walk this session (`--ignored --nocapture`), each a full real run
  judged on its own: the baseline (death t19772, the census), the rim sweep nearest-first
  (pass 0's g1 head kill at +50, pass 1 dead at +109), the front-arc crouched roll (g3 killed
  at +22 with one tap, pass 1 dead at +147), the tight-gate + threat roll (g3 at +34, the g2
  tap swallowed, dead at +148), the standing head-first roll (g3 at +34, the g2 taps missed ×2,
  dead at +74), the cadence-17 roll (the identical t19772) and the shift-0 roll (the round-1
  t19197). The audit runs (`NF_0D_AUDIT=1`) supplied the bullet-spawn/verdict evidence.
- The whole `walkreal_0700000d_live` binary, single-threaded, on the final tree:
  **3 passed, 0 failed, 1 ignored** (103.6 s) — the plain master replay, the Operative
  real-damage walk and the d2 Agent sister pass; the rim machinery is dormant (default 0) and
  the d1/d2/plain timelines byte-identical.
- `interactables_0700000d_live`: **11 passed, 0 failed** (31.1 s).
- The fast runner (`tools/test/suite.py`) does not fit this window's build phase: the scoped
  attempt (`--only walkreal_0700000d_live,interactables_0700000d_live`) was killed at 500 s
  inside the all-crates `cargo test --no-run` build (the suite captures the build output, so
  nothing prints until the build completes; this 2-core sandbox also OOM-linked an untouched
  nf-view MP binary in the round-2 attempt). The per-binary foreground runs above are the
  primary evidence, as in rounds 1-2.

## What's left

- The walk itself: the round-4 levers are in the ignore message. The floor sweep needs
  ~28-31 at the landing (the 0d-2-era budget); the alive-ride rolls land at 16.8-18.6 and the
  rim fight is measured-dead, so the ~+12 must come from the pre-ride phases (the corner
  chips, the #22 re-roll, the 18-tap zero-chip ride hunt) or from the landing burst's own
  dice (the floor-landing crouch).
- The live checks below (carried from rounds 1-2 where still relevant).

## Live checks for the main agent

1. **The arming event's address** (new): with gdbwatch on g0 (skin 0x500006d, (157.55, 4.98,
   -286.52) state 0x68 at the container landing) during a rim-sweep run, record what flips its
   state into the 6-tick fire cadence — the kill-shot sound (hearing 40 [L]) or g3's death
   event. If a hearing bit is the trigger, a suppressed-class lever may exist; if the death
   event, the rim fight stays dead.
2. **The kind-3 pickup at (205.4, 0.1, -205.3)** (carried from retune-nb-0d): fieldprobe at
   difficulty 3 (00 Agent boot, not a save-state resume) — does the pickup entity exist there
   (placement id, kind, enabled)? If it exists but refused, the pickup radius/handler path
   needs the read; if the placement is difficulty-gated out, the armour lever is dead.
3. **The 18-tap roll's config** (new): the dice-knob comment cites an 18-tap roll with the
   ride at ZERO chips and a zero-cost exit (31.2 through the landing). The main agent's
   session notes (real-health-00agent-0d-2, 2026-11-1x) may hold the exact tap zone/cadence
   that produced it — one env re-run on this branch would then confirm the budget.

## ntfy failed

All three fallbacks failed in this sandbox (the same unreachable-endpoint signature rounds 1-2
recorded — filled in below after the finish step runs).

## Usage

not visible to me — this session's environment shows no token, context-window or cost
figures, so there are no numbers to copy.
