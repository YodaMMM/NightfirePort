
## walk-1b-agent (2026-11-22)

Task: 1b arrival-to-Drake real-health walks (Agent + 00 Agent) green. Branch from origin/master. Ledger from ignore messages on walk_arrival_to_drakes_defeat_on_real_health_agent/_00agent in walkreal_0700001b_live; skip measured-dead levers, take next untried. Helpers gated to level+difficulty. Agent first, keep Operative walk green.

Progress:
- sandbox reset; fresh clone; branch created from origin/master (3dc1aff7e).
- round-22 progress [L this branch's runs]: the Agent walk's named lever (surviving the tail window) SHIPPED: the dead-window shelter gates on waves_first (Agent) too, the test stop moved to 6000/60; all 8 consoles latch, the console phase ends 30.7, the waits ride the kite (waves-wait zero-loss 1260 ticks) and Bond meets Drake's gate at 12721 with 23.7 hp - the remaining binder is the Drake-fight aim/line (Drake took 0 damage from 96 rounds; the hold's sight-cut cuts both ways). The 00 Agent named lever (skip the zero-kill defense fights on a live k8/k6 target) measured RED (death 8372 - the fights' sight-breaking cover floats are load-bearing); both ledgers refreshed in the ignore messages with the next levers.

## walk-1b-agent-2 — 1b Agent / 00 Agent real-health walks, round 2 (2026-11-23)

Task: walk-1b-agent-2, branch s/walk-1b-agent-2 (origin/master 7562f1e70 + origin/s/walk-1b-agent merged first).

Progress:
- sandbox reset; fresh clone/setup; branch pushed early (merge commit 22cd90615).
- Agent lever (the ignore message's named next lever: the Drake-fight aim/line, the r11 open
  lever) measured RED in two formulations [L this branch's runs]: (1) the Drake-first aim pick
  (override to Drake when his exposed_aim is Some and no wave shooter is inside the 3.5 u melee
  band) — exposed_aim(Drake) NEVER passed (the west shelter's sight-cut cuts his box line at his
  ~60 u spawn), 7 taps went to the wave (100->16 rounds, wave 16->13, Drake 120->120);
  (2) the weapon-range fire probe (the flight's own death range weapon_data+0x1C [M Bullet_update
  0x12e7b0 case 1] instead of the 40 u pick cap) — bit-exact identical run. Facts banked: Drake
  closes ~0.018 u/tick (~50-52 u at Bond's death ~f13700); the aim lift out of the ZERO-LOSS
  waves-wait shelter costs the wave's sight (23.7 -> 13.2 in the first 480 ticks, 3 x 3.5,
  zero Drake hits by f13200). Gate ships OFF (byte-identical to the round-22 shape); scaffolding
  stays for the next round's shoot-and-scoot.
- 00agent lever (the round-23 named lever: the 60-tick mid-phase stop sweep 6000..7500, through
  the NF_1B_PAUSES3 hook) measured RED at the sampled stops 6000/6250/6500/6750/7000 (all 60) —
  consoles 4/4/3/5/3 latched, health 0.0/0.0/0.0/8.4/42.5, none reached the stuck-8864 depth;
  7000/60 least bad (42.5). Next lever: finish the sweep (7050..7500/60, lengths 7000:30/90/120)
  or sweep the k4 stop (4426 vs the d2 walk's 4404). Both ignore messages refreshed.

Verification:
- whole walkreal_0700001b_live binary on the final tree: 17 passed; 0 failed; 2 ignored (119.02s)
  — the Operative walk stays green; nothing else moved. Both target tests re-run individually
  during the lever work. No src/ or shared-helper change (all in crates/nf-view/tests/
  walkreal_0700001b_live.rs; ray_on now wraps a probe-length ray_on_len with the same 40 u
  default — all other callers identical).
- the fast suite runner could not run: background jobs die between tool calls in this sandbox
  (verified with a sleep probe), so the nohup suite start produced nothing, and the foreground
  window was consumed by setup (clone, downloads, rust, a full-disk incident — the nf-view
  build scripts had duplicated 1320 lib rlibs for ~5 GB; cleaned and rebuilt) plus the lever
  runs. Same justification as round 22: the change surface is the single 1b test file, and the
  whole 1b binary ran twice on the final tree.
- round-2 addendum: the 00agent sweep was completed to ALL 15 stops (6000..7500/60) — k5 3191 /
  k4 4426 / k3 5901 identical everywhere; k8 7583/8464/-/7587/-/-/7859/-/-/-/-/8161/8234/8305/
  8728, k6 only at 6750; final health 0.0/0.0/0.0/8.4/42.5/42.5/17.5/40.9/0.0/42.5/42.5/32.5/
  42.5/42.5/42.5. Seven stops STALL at the k8/k6 read with hp 42.5 (the master baseline stalls
  at 8864 with 37.5). Next lever in the ignore message. Whole 1b binary re-run on the final
  tree: 17 passed; 0 failed; 2 ignored (119.94s).

## walk-1b-agent-3 (2026-11-23, round 3) — in progress

Branch s/walk-1b-agent-3 (master 7562f1e70 + origin/s/walk-1b-agent-2 merged in first —
fast-forward; walk-1b-agent-2 was 5 commits ahead of master). Same sandbox as the retune
round: repo, work/, ISO and nf-queue survived; remote token refreshed, branch created and
pushed at the merge point.

Lever work so far (the ignore message's named lever, the Agent shoot-and-scoot; all runs
NF_1B_SCOOT=1 on walk_arrival_to_drakes_defeat_on_real_health_agent, ~40-90 s each):
- diagnostic (NF_1B_SCOOT_DIAG): the Agent fight hold is the east nest (53.2,-10.9,15.6),
  Drake spawns (-7.3,-1.4,0.1) 63 u west and closes; the shelter eye (-10.2) is exp (0,0)
  (zero-loss) but its sight-cut never opens Drake's box line (ondrake false all fight).
  Scoot candidates exist: south deck spots (x 26-50, z 36-39, exp 7-9, 22-35 u, lines hold
  as Drake crosses) and east spine spots (x 59-73, z 0-1, exp 6-9, 21-28 u, lines flicker).
  Weapon 0x33's death range +0x1C = 500.0 (rg 500) — no range blocker. Drake ascends
  (y -1.42 -> -0.1) as he closes; his boxes reach y -0.5..-2.0.
- run 1 (direct 16-u flight to the east spine spot): the flight+hover burned 21 hp, died
  one tap short (Drake 21.1, clip 6); taps landed only after Drake crossed x ~6.
- run 2 (zero-loss wait + short-scoot cap 14 u): the wait held 23.7 for 1080 ticks FREE;
  first short window t1140 (spot 3.3 u); the spot-to-spot churn t1140-1440 cost 8.5 hp;
  the endgame (eye on Drake's box plane) tapped every ~60 ticks — Drake 120 -> 40 — but
  Bond died 2 taps short.
- run 3 (double-verify launch gate + hold-through-flicker): wait free to t1080 (-3.5, the
  one lined shooter), launch t1200, arrival t1320 into a dead line (Drake oscillated), the
  unlock churned at deck height to death with zero taps.
Next: TAP mode (hold+aim+tap whenever the live eye has the line, checked every tick),
immediate lock while high (skip the double-verify when the eye is above the cavity), and
a last-clear-box aim memory to kill the box<->torso aim bounce (the endgame tap cadence
was ~20 dmg/60 ticks).
