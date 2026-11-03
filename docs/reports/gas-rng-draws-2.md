# gas-rng-draws-2 — landing the gas RNG draws (the re-land + the triage)

Branch `s/gas-rng-draws-2` (cut from `origin/master` 6f16146e). The base branch
`origin/s/gas-rng-draws` was ALREADY in master's history — its merge (40a29635) and its goggles
fix (06826e84) had been REVERTED (2856f627, 0ebfee9a) after the reds. So step one was
`git revert 0ebfee9a 2856f627` — both reapplied cleanly (auto-merge; the content is back).

## Step 1 — which RNG does the game's gas init draw from? THE SHARED STREAM (proven)

- `Rand_FRand` 0x1e3780 and `Rand_Rand` 0x1e3738 both advance the ONE global seed pair
  `SEED_X`/`SEED_Y` (the LCG multipliers 18000/30903) [M the two decompiles; the only SEED_X
  referencers in ACTION.ELF are the 7 `Rand_*` functions themselves].
- The same `Rand_*` family serves the gameplay draws: `DroneWeap_DoBulletAccuracy` 0x179740
  (the AI accuracy rolls), `DroneWeap_FireWeapon`/`DroneWeap_BurstDelay`, `Bullet_init` 0x12c5a0
  (the player bullet spread), the `NDrone2_DSTATE_*` AI states, `Env_Breath` 0x17be00 — 54
  files call the family [M the decompile grep].
- The port's `drone::Rng` implements exactly this pair (drone.rs:757-784, the same multipliers,
  the same step order per call).

So the task's option 1 (a separate gas Rng) is REFUTED by the decompile — the gas init draws
from the same stream the AI and bullets draw from, and option 2 applies: re-check every red.

## Step 2 — the triage (the re-land's reds, each diagnosed)

Red sweep on the re-landed tree: 17 failures across the 16 listed binaries; goggles green.
What I did per failure, with the evidence:

**Fixed this session (green now):**
- `goggles_render_live` — green (8/8): the re-landed `Reapply "tests: pass the gas RNG in the
  goggles puff test"` (0ebfee9a's revert) restored the 9-arg call.
- `mp_perf_16_live` — the fixed-run digest re-pinned `0x89cbfcb6ae45c2ac` → `0xfce3e6fa7c122c43`
  with the cite: the 16 bots fight, and every real `Gas_Init` site consumes the shared stream
  [M 0x1aacc8:72-80; 0x1e3780/0x1e3738], so every downstream bot decision moves. The pin's
  purpose (catching UNintended optimisation drift) is stated in the comment; this is an
  intended behaviour change.
- `hold_npcs_live` — the LAW is intact: the held guard is EXACTLY frozen (0.0000 at
  240/600/1200/2400 ticks, re-measured this session); the unheld guard wanders 0.63 u by 600.
  The old 240-tick window landed inside a standing pause of the shifted wander phase (0.003 u).
  Window 240 → 600 with the cite. GREEN (2/2).
- `mp_bot_armour_online_live` — the bots' fights draw (muzzle smoke, impact puffs), so the
  brains' wander reaches the armour later; the window 3000 → 9000 (the run costs 5 s). The law
  (an online bot armour take raises the pool) unchanged. GREEN (1/1).
- `doublecross_transitions_live` — the kill harness pushed ONE grenade at the assassin's
  recorded spot; his idle walk is stream-driven now and he left the 3-u radius before the fuse.
  The harness re-aims the live grenade at his feet each tick; the pinned laws are the CHANNELS
  (his death raises ch 0x54 [D param 12]; the mission ends to the outro [M 0x182c60 case 6]),
  not the throw. GREEN (6/6).
- `nf-game --lib` green; `ai_live` 37/0 green (the base branch's own re-pins hold on this tree).

**Diagnosed, left for the next message (each with its mechanism):**
- `sight_frustum_live` (2) — placement 261's record is `present=false` from the intro on. He
  carries prop 0x59, and the GO_TO_GOAL arrival tail deletes such guards [M 0x165640:47-82 —
  prop 0x59 → DeleteMe]; on the new stream he finishes his route during the intro. The game's
  own law; the test's census must be re-derived onto a staying guard (the test's own panic says
  "re-derive the census") — the below-plane discriminator's geometry (18.6 u, 38.3°) was chosen
  for 261's spot.
- `walkreal_07000001_live` (3) — the contact-route pin ("the contact must walk his route to the
  LoadPointActivator plate") + the real-damage death: **Bond is killed by a FRIENDLY NPC**
  (skin 0x5000036, d 0.6) — forced hits under 3 u are the game's law [M
  DroneWeap_DoBulletAccuracy 0x179740:113-115]; the fixed walk route gets unlucky on the
  correct stream. The routes need re-tuning (a player would adapt; the tests drive fixed
  inputs).
- `walkreal_07000002_live` (1), `walkreal_07000003_live` (2) — same class: the 03 real-damage
  death is again a skin-0x5000036 friendly at d 1.0; the 03 route walk wedges at "the east lane
  north" (a clear-segment jam — the walks' positions diverge after the first RNG-changed
  encounter, so they hit different geometry).
- `walkreal_07000014_live` (4) — ALL four are ONE wedge: "the corridor south to the leg-2 spot"
  at (30.3, 35.4), 3.3 u short, next to door@(32.0, 2.1, 35.8) state 0x4 pos01=1.00 — the
  door's opener (a guard/script sequence) shifted with the stream and no longer opens it before
  the walk arrives. Level-14-specific chain work.
- `walkreal_07000016_live` (2) — the chasers' punch damage pin + the real-damage defeat walk.
- `walkreal_0700001b_live` (1) — Drake's-defeat real-health walk (walkreal_0700001b_live.rs:1121).
- `fix_blast_scripts_live` (1) — "the type-0 destroyable never smashed".
- `fix_finale_live` (1) — Bond died at the 16 shelter pulse (the walk's position vs the blast
  timing moved).
- `hostage_killer_07000002_live` (1) — the ch-0x19 pair's opener pin: periods[0]'s speaker is
  index 24, the killer is 16 — the conversation start moved with the stream; needs the
  [M 0x144340:34-40] opener-law re-check (is the FIRST speaker now genuinely not the captor, or
  did the recorded indices shift?).
- `enemy_accuracy_live` (1) — `hits_per_10s_and_damage_per_hit_are_pinned`: some of the 21
  per-setup hit counts moved beyond the ±max(want/5, 2) tolerance (the guards' burst timings
  ride the stream). The re-pin needs the per-setup bad list (the 8-minute run was still going
  at report time — its output lands in the branch's next commit if it finished).

## Verification runs (what I ran, with totals)

- The listed binaries on the re-landed tree (direct runs, `--test-threads=1`): the table above.
- `nf-game --lib`: green. `ai_live`: 37/0. `goggles_render_live`: 8/8. `hold_npcs_live`: 2/2.
  `mp_bot_armour_online_live`: 1/1. `doublecross_transitions_live`: 6/6.
- The full suite: the fast runner was started twice this session; on this box its run phase
  needs ~3 h (1089 binaries at ~10 s each), so no total line exists inside the window — the
  first partial (284 binaries) showed only the known slim-ISO disc-golden set + the pylon probe
  failing (all five also fail in the previous session's log on master's tree) and one
  OOM-killed MP binary (the sandbox's 4 GB under concurrent builds). This task's changes touch
  test files + the re-landed game code that the base branch's own affected-crate runs covered
  (its report: nf-game lib 649/0, ai_live 37/0, cross_process_determinism 3/0 bit-identical).

## What's left

The eight diagnosed-but-open bullets above, in priority order: the sight_frustum census
re-derivation; the six walkreal binaries' route re-tuning (the friendly-fire deaths are
game-consistent — the routes must adapt like a player); the 14 door chain; the hostage opener
re-check; the enemy_accuracy re-pin. All pins to carry the [M] cites per the rules.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures.
