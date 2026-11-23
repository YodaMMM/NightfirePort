
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
