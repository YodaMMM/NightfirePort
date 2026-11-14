# level-0d-operative-walk-7

Branch `s/level-0d-operative-walk-7` (from origin/master 385ebb66d). Task: take the
walk-6 ignore ledger's named levers in order on
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`, each measured on its own
real run; real input only, no top-ups, never lower damage or health; keep the plain (top-up)
and d2 walks byte-identical.

## Status: still red, the ledger advanced materially

The walk still dies in the ladder-top phase, but the death moved **3733 → 4514 (+781 ticks)**
and the phase landed **two new kills** (#16 and #18) that had never landed on any roll. The
baseline reproduced the walk-6 v5 roll exactly (death 3733 at (213.07, 17.37, −234.51), 10
hits for 100.0, 0 return-fire fights: #29 48.0, #16 32.0, #28 14.0, #10 6.0, #18 0.0).

## What was measured (seven real runs, in order)

1. **Lever (a), the pre-climb band stand from the ladder base — MEASURED DEAD, leg removed.**
   The ledger's hypothesis was right that the line EXISTS: #17 spawns live and enabled at
   (158.43, 1.00, −216.48) and the base→band ray printed full CLEAR / static clear /
   bullet_scoped CLEAR at d 56 while she patrolled her east corridor (z −218..−223; the -5
   strip-mouth 0.975 law is only the z ~−225+ west segment). But the act never landed in four
   runs: the aim cursor pins ~4-5 u off a far off-centre target (cross stuck 0.09-0.29 across
   the raw act, the drained act, the no-drain act, and the d2 240-tick pre-aim tracking law)
   while her 0.042 u/tick patrol crosses the ~120-tick corridor before any convergence; the
   ladder-top band arm's rays to her west and NE corners die 0.967-0.988 from the top too.
   Costs: ~500-800 ticks of dwell per roll, 14.0 landed across the re-rolled dice on the
   fifth run, and every death got WORSE (4251/4571/3934/3804 vs 3733). The #18-leg precedent
   applies ("the net is NEGATIVE: the leg stays out") — removed, with the full law in a
   comment at the climb and in the ignore message.
2. **Lever (b), the deck act from the base — MEASURED DEAD in that form.** The pick finds #16
   in-box (198.95, 14.04, −260.41, state 0x68) but the up-hill ray from the base (eye 1.8)
   dies **0.040 along on the crane's own structure** — clear_box_aim returns None, zero
   rounds in 400 ticks. The d2 #20-leg geometry does not transfer to #16's line.
3. **Lever (b)'s real form — LANDED: the deck #16 box act's FIRST turn at the ladder top**,
   before the retarget loop, real-damage-gated. Down at **+209 ticks** (the ledger's
   "+209 twice" law verbatim), ray full CLEAR from the ladder-top eye. Removing her takes her
   32.0 off the phase below, and the phase's later deck leg self-cancels.
4. **Lever (c) extension — LANDED: the bay-duo arm → Operative** (the d2 t4312 drain law's own
   box [209.3, −289.5] ± (7,3)). #18 down at **+195** on the first pass. #10 survived two
   passes under converged CLEAR-ray aim (cross ~0.0) — see the hip-fire law below.
5. **The Operative band arm's caps trimmed 600 → 200**: the top rays to #17 are blocked every
   roll (0.967-0.988 measured twice) — the 600-tick caps were pure cadence exposure under the
   roof pair's fire. The d2 twin keeps its 600.

## The new floor and the next levers (written into the ignore message)

- **#27/#28 are ACT-PROOF from the top on this roll** — pick_trio_target returns no torso box
  and the retarget loop breaks by its own law. The head-line gamble is the only in-phase act
  left (the run-4379 graze law says it is pure exposure; measure once before believing it).
- **THE OPERATIVE HIP-FIRE LAW** (measured this session): the aim bit flaps with the fire
  script between shots, and the Operative path's trigger gate (`sim.aiming || !agent_fast`)
  always passes at d1 — so acts against #10/#27/#28 fire guaranteed-miss 100-unit-cone hip
  rounds even with the ray converged and cross at ~0.0. **Next lever #1: thread an aim-bit
  trigger gate through shoot_far_aim gated on difficulty == 1 && real_damage** (the plain
  replay and the d2 twin must stay byte-identical). This converts those acts' taps to the
  aimed act and is the direct fix for the #10 drain (38.0 in the pre-bay-duo roll).
- Then: the yard levers (the fast first tap, the girder sweep, walk-5's wired set); the
  port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] stays its own task.

## Verification

- `walkreal_0700000d_live` whole binary: **2 passed, 0 failed, 1 ignored** — the plain
  top-up replay and the d2 real-damage twin are green and byte-identical (all new legs gate
  on `difficulty == 1 && real_damage` or keep their existing gates).
- `interactables_0700000d_live`: **11 passed, 0 failed**.
- The ignored Operative real-damage walk: still dies at tick 4514 in the ladder-top phase
  (measured on its own run each time; 10 hits for 100.0, 0 return-fire fights).
- The full suite did not fit the session: the sandbox was reset at session start (repo, ISO
  and toolchain gone), so setup + from-scratch build consumed the first ~45 minutes. The
  task-mandated binaries both ran whole. What's left for the next session: the fast-runner
  full suite after the aim-bit lever lands.

## What's left

The walk is not green. The next session starts from the ignore message's OPERATIVE-WALK-7
LEDGER: lever (1) the aim-bit trigger gate, then (2) the act-proof pair's head-line gamble,
then (3) the yard levers.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).

## ntfy failed

All three fallbacks failed in order:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-0d-operative-walk-7: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...'), timeout=25)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-0d-operative-walk-7: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out and was killed after 60 s (no output).

