# 0700000d: the Agent real-damage walk dies at tick 3310

The ignored Agent real-damage walk on 0700000d dies at tick 3310, before the wire. Find the guards and rounds that kill Bond there (damage log), compare their accuracy, engage range and damage with the game's rules for those guards, and fix the port if it differs; otherwise re-route as a player would and un-ignore it if it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-agent-real-damage`, report in docs/reports/level-0d-agent-real-damage.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-08T22:38Z

---
## Report (done, 2026-11-08T23:26Z)


# level-0d-agent-real-damage — the t3310 death at the ladder top: killers found, the port's laws compared clean, the re-route pushed the death 3310 → 4275

## What the task was

The ignored Agent real-damage walk on 0700000d
(`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage_agent`) died at tick 3310,
before the wire. Find the guards and rounds that kill Bond there, compare their accuracy, engage
range and damage with the game's rules for those guards, fix the port if it differs — otherwise
re-route as a player would and un-ignore the walk if it passes.

## Done

1. **Reproduced through the real input path** (no teleports after the load; `--ignored --nocapture`
   on `s/level-0d-agent-real-damage` at 66500c1bc): Bond died at tick 3310 at the ladder top
   (213.07, 17.37, −234.51), armour spent, **6 hits for 100.0, 0 return-fire fights**:
   - 2395: −0.00 and 3047: −6.00, #31 = placement #461 south west-roof (skin 0x5000090, sniper
     0x1e) at (150.9, 22.7, −248.4), d 64.0;
   - 3047: −28.00, #20 = placement #360 the deck guard (0x5000087) at (198.9, 14.0, −260.1), d 29.5;
   - 3048 + 3228: −28.00 ×2, #32 = placement #462 north west-roof (0x5000087) at (151.4, 22.7,
     −248.5), d 63.5;
   - 3236: −10.00, #11 = placement #292 the bay captain (0x5000093) at (214.7, 12.1, −289.5), d 55.3.
   The route's own kill boxes were laid for the -5 tree's spots ((151.4,−241.3)/(151.1,−255.8)):
   all three roof snipers settled 4.25–7.3 u outside them, so every ladder-top leg read "no
   PRESENT guard near" and Bond stood in a six-sniper field with nobody dying.

2. **Compared the killers' accuracy, engage range and damage with the game — the port matches;
   there is no port fix to make on this branch.** Every term checked against the decompile:
   - *Hit chance* — [M DroneWeap_DoBulletAccuracy 0x179740:49-115]: `p = 100 − 5·stat`, the
     placement stat 4 → 80 [D 0700000d-audit kind-1 words], TooClose ×2 (d < 3), first-moved
     ×0.25, first-stopped ×0.1 on 0c/0d for anim-set 0x13 (the Winter snipers), moving ×0.75,
     Agent (GameState+0x28 = 2) ×Accuracy_Normal 0.8, the 120-tick continuous-sight gate, hit iff
     `Rand_FRand(100) < p`. The port (`drone.rs:12724-12740`, called at `:11846-11856`) is
     term-for-term identical, with the bit-exact `Rand_FRand` (:782-790).
   - *Engage range* — the west-roof placements are vision class 4 [D audit p14=4]:
     [M NDrone2_DefaultInit 0x14b300:380-424] gives sight 1000, engage 75+FRand(8), combat 1000;
     the mode-5 sniper states (SniperIdle 0x2A → SniperAim 0x2B → SniperFire 0x2C) fire on their
     timer with **no distance check at all** [M 0x15fd88:34-63; Ready2Fire 0x178c48 is a gun-angle
     gate]. So snipers shooting Bond from d 55–64 IS the game's law. The "48 u window" in the
     route comments is the test harness's return-fire ring (`damage_log.rs:474`), not an AI rule.
     The port mirrors all of it (`drone.rs:11244-11353`, `:8161-8196`, `:11713-11755`,
     `:11950-11964`), including `DroneFunc_SetAsAttacking`'s cone zeroing and sight raise
     ([M 0x14aed8:25-26] — `+0xe4 = 0`, `+0xe8 = +0xfc`; port `first_attack_sight`).
     The phase was "free" on the -5 tree because that tree predated the class-4 sight table
     (landed in 2998b6a76, 2026-11-08) — the old behaviour was the deviation, not this one.
   - *Damage* — [M Player_HandlePain 0x1902c8:29-49,100-134]: base `weapon_data[0x1e]+0x0C` =
     40.0 [L ee_start_05.bin], × Plr_DMod_Normal 0.7 on Agent → **28.0 per unarmoured hit**; no
     distance falloff, no variance, no SP body-part multiplier. The 10.0/6.0 deltas are armour
     absorption (`absorbed = min(damage, armour)`), and 0x5000093 is the captain skin
     ([M 0x14b300:556-562]: ×1.5 → 42 base). Port matches (`drone.rs:11894-11898`,
     `:12742-12758`, `bullet.rs:415-427`). Two cosmetic deviations found while reading, neither
     load-bearing here: the port skips `FireWeapon`'s `Rand_Rand(500)−250` shot-pitch draw
     (documented, `drone.rs:11899-11904`), and the game truncates the armour absorb to
     `(int)damage` — zero effect for integer 28/42, real for fractional weapons (a flagged
     follow-up, not this task).

3. **Re-routed as a player would** (test-only changes in `walkreal_0700000d_live.rs`):
   - the ladder-top **head sweep** (one wide box over the whole wake band x 145–157, z
     −236..−258, the box act on a real posed head-box centre) replaces the stale fixed boxes —
     first pass killed a roof sniper at **+248**;
   - the **#17-first leg** moved directly after the climb — ANY ladder-area shot wakes him (his
     fire spot is d 39, inside the P2K hearing radius 40 [L]), and his box also takes the deck
     guard: **deck #20 down at +209**;
   - the **bay wide box** ((209.3,−289.5) ± (7,3), the guards wander 203.5–214.7): the captain
     down at **+349**, the annex sniper #363 down at **+133** (three passes);
   - the aim-drain **fast path** in `shoot_far_aim` (skip the 160-tick drain while the aim bit
     holds) — every later pass converges ~250 ticks instead of ~500.
   Result: the death moved **3310 → 4319 / 4275** across runs, with the deck, the bay pair and
   one roof sniper proven dead before it. The remaining ledger is ONE guard (below).

## The one remaining blocker (measured, named)

The deep-band roof guard #463 settles at (151.4, 22.7, −248.5) in the firing stance (state 0x97)
where **the head line from the ladder top dies 0.974** — the -5 "head clears where chest dies"
law holds only north of about z −245.5 (this run's own mid-sweep trace: the line to
(151.1, −250.8) blocked 0.974). The box act correctly held fire every pass, she fired on the
180-tick cadence at 64 %/shot, and 4 × 28 + the killing 10.0 ended the run. The pre-wake snipe
is measured dead: the west roof **spawns at the platform-drop wake** — there are no bodies before
it (the probe's CLEAR pre-wake rays are abstract points, not live guards).

Next levers, in the order I would try: (a) skip the lost duel — jump-grab the wire and take her
from the deck after the ride (her line to the rider/deck is unmeasured); (b) longer sweep caps to
catch her patrol crossing a clear-ray spot (the -5 g29 south-walk law); (c) the port-side
standing capsule in `drone_ray_part` [M NDrone2_Collision 0x140e90:91-121] so chest lines land
on fire-anim guards the way the game's do — that is the wire-jump-grab report's owed fix and it
would unblock the whole chest-act family, but it shifts every walkreal fight and needs its own
session with every `walkreal_*_live` re-run.

## How it was verified

- `walkreal_0700000d_live` whole: **the plain twin green (1 passed, 2 ignored)** with the new
  legs in the shared body — the ride, deck, plate and annex asserts all still pass.
- The Agent sister re-driven 4× (`--ignored --nocapture`), ledgers above; each run 8–10 s.
- No `src/` changes at all: the branch touches the test file, `docs/WORKLOG.md` and this report.

## What's left

1. The Agent walk stays `#[ignore]` (it does not pass yet — the ignore note carries the fresh
   ledger and the next levers). Un-ignore when the #463 lever lands and the walk goes green.
2. The flagged port follow-ups: the `(int)` armour-absorb truncation in `PlayerHealth::apply`
   (fractional weapons only), and the standing-capsule `drone_ray_part` fix (with the full
   walkreal re-run its own session requires).
3. The Operative sister's ignore note describes the pre-2998b6a76 tree; its route economics
   changed with the sight table — a later session should re-measure it the same way.

## Live checks for the main agent

1. **Class-4 sight at long range on hardware.** In PCSX2 on 0700000d (Agent), stand at the
   ladder top (213.0, 17.4, −234.5) after the platform drop and wait. Expected: the west-roof
   snipers at d ~63 acquire and fire from there (sight 1000, no distance gate in the sniper
   states [M 0x14b300:380-424, 0x15fd88:34-63]) — the port now behaves the same. A gdbwatch on
   one roof guard's `+0x270` (sight ticks) during the approach should show it filling while Bond
   is still on the strip.
2. **The deep-band head line.** From the same stand, aim at the guard the port measured at
   (151.4, 22.7, −248.5) and fire. Expected on hardware: rounds land when she steps north of
   ~−246 (head line clears) and die on the roof slab edge at −248.5 — confirming the 0.974
   blocker is world geometry, not a port ray bug.
3. **Does a landed first hit really zero the cone?** Poke one roof guard's `+0xe4` (half-FOV)
   after its first shot. Expected 0 per [M 0x14aed8:25-26]; if it holds π/2 on hardware, the
   port's `first_attack_sight` cone-zero is over-broad and the acquisitions need re-checking.

## Usage

Not visible to me. This sandbox shows no token, credit or context-window figures for the session
— no meter, no usage dashboard, nothing printed by the tooling I can read. The per-subagent token
counts returned by the two research agents (291,276 and 177,382 total tokens respectively) are
the only numbers I could read, and they are subagent calls, not this session's budget.
