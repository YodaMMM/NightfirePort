# 07000002: the courtyard sniper's hits on Bond

On the castle walk, crossing the courtyard at the stair bottom, the courtyard sniper lands hits of 13.7, 28 and 41.7 on Bond (Agent). Read the sniper's weapon record (damage, accuracy, difficulty scale, range falloff) and its fire rate in the decompile, measure the port's hit chance and damage per round at 15-25 u against what the game's code gives, and fix any difference. Pin the per-round damage and the hit rate at one range through real input.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-02-courtyard-sniper`, report in docs/reports/level-02-courtyard-sniper.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-08T15:11Z

---
## Report (done, 2026-11-08T16:02Z)


---
## Report from the worker (done, 2026-11-08T16:02Z)

# level-02-courtyard-sniper — the courtyard sniper's rounds on Bond (07000002, Agent)

## What was asked

On the castle walk, crossing the courtyard at the stair bottom, the courtyard sniper lands hits
of 13.7, 28 and 41.7 on Bond (Agent). Read the sniper's weapon record (damage, accuracy,
difficulty scale, range falloff) and its fire rate in the decompile, measure the port's hit
chance and damage per round at 15-25 u against what the game's code gives, fix any difference,
and pin the per-round damage and the hit rate at one range through real input.

## What the game's code gives

- **The weapon record.** The courtyard sniper carries weapon 0x1e (the AI L96A1): `weapon_data`
  0x2bf150 record 30 = damage **40.0** (+0x0C), 1 pellet (+0x18), flight range 1200 (+0x1C),
  speed 13.2/tick (+0x20), spread 100 (+0x24) — dead-on, because flags +0x68 = 0x408050 carry
  bit 0x400000, the zero-spread gate (`bullet::spread`, `Bullet_init` 0x12c5a0:139-170) — burst
  1 (+0x2C), interval 120 frames (+0x40) [D weapon_data, dumped through the port's boot
  initialiser]. There is **no range-falloff field anywhere in the damage chain**.
- **Per-round damage on Agent at 15-25 u = 28.0.** The chain: 40 × `Drone+0x100` 1.0 (the
  sniper is no captain, not a 0x63-0x66 weapon) [M `Drone_ModBulletDamage` 0x139f50:21-27] ×
  Plr_DMod_Normal 0.7 [M `Player_HandlePain` 0x1902c8:82-95 with the level 0x01..=0x04 tuning
  group at 0x1d3e00]. The only other modifiers are TooClose ×2 (under 3.0 u — the stand sits at
  21 u) and the back-shot ×2 (mode 2 + anim set 0x12 + level ≠ 0x1B — the sniper's anim set is
  0x13, so the gate can never fire for him) [M 0x139f50:24-48]. `Player_HandlePain`'s
  head/limb table (Plr_DMod_Head 4.0, limbs 0.8) runs ONLY behind `MPSettings._384_4_ != 0` [M
  0x1902c8:82-104] — single player never applies it. Cross-checks: the 00agent ledger's one
  25.7-u hit landed 48.0 = 40 × Plr_DMod_Hard 1.2 [L the 00agent-02-03-2 ledger]; the 07000005
  trace's sniper hit 18 = 30 × 0.6 on a different level's record [L drone-ai.md].
- **The armour split explains 13.7.** `Player_HandlePain` drains BL+0x8B0 before health [M
  0x1902c8:84-104]: a 28-round into armour A < 28 takes A off the armour and 28 − A off health.
  13.7 = 28 − 14.3 — a 28-round into 14.3 armour. The pin's own run passed through armour 14.3
  at exactly this crossing [L the pin run's armour timeline: shots 8-9 sampled armour 14.3].
- **41.7 is the run's cumulative, not a hit**: 13.7 + 28.0 = 41.7 — the total after the next
  full round lands on bare health.
- **Hit chance at 15-25 u.** `DroneWeap_DoBulletAccuracy` 0x179740: (100 − 5 × accuracy 8 [L
  the 07000002 drone dump]) = 60, × DroneFiring_Accuracy_Normal 0.7 [M :74-88 + the tuning
  group] = **42 %/shot on a standing, sighted Bond**; 31.5 %/shot moving (× TargetMoving 0.75
  [M :78-80]); the first-2 s of sight miss outright (NewSighting_TimeToHit 2.0 × FRAME_RATE =
  120 ticks of +0x270) [M :93-99] and so does everything while +0x41 stands (the unseen arm
  sets it, fire clears it [M 0x176a28 unseen arm + FireWeapon]); misses still fly a wobbled
  round [M :116-134].
- **Fire rate.** Burst 1, interval 120 frames, between-burst delay
  max(15, trunc(45 × 1.33 × (0.5 + FRand 0.5))) ≤ 60 at 15-30 u (aggression 1) [M
  `DroneWeap_NextBulletTime`/`BurstDelay` 0x179d40] — one shot every 149-179 frames while
  continuously firing.

## What the port does — measured, not assumed

The port's code paths (`hit_chance` drone.rs, `player_hit_damage`, `PlayerHealth::apply`,
`fire_weapon`) implement exactly the chain above — verified line-by-line against the decompiles
today. The measured behaviour, through the new pin test
`courtyard_sniper_pin_stair_bottom_agent` (crates/nf-view/tests/walkreal_07000002_live.rs):

- **The route is real input from the spawn** (no teleport after the load): the spawn-room body
  armour taken, the garden/south-hall fights answered with real fire, the geisha-1 captors
  cleared through the real fire path, then Bond walks to the stair bottom and STANDS with
  return fire off. The sniper (drone #9, weapon 0x1e) covers the stand from his post at
  (−52.05, 6.34, −14.43) — dxz 21.3 u, inside the 15-25 u window [L the drone dump].
- **Damage per round = 28.0, pinned by landing.** The stand took 15 shots; 3 rolled hits
  landed: **27.35 health + 0.65 armour** (armour 0.65 standing at the shot), **28.0** health
  (bare), **28.0** health (bare) — every round totals exactly 28.0. The test asserts it: never
  more than 28.0, exactly 28.0 into empty armour, and the armour part never exceeding the
  armour present.
- **Hit rate = the formula's.** Per-shot the gate inputs print (seen_ticks, lost_sight, pct):
  the first two shots were gate-blocked by the game's own 120-tick NewSighting window (seen 39
  and 34), the rest rolled at pct 42.0; 3/13 gate-open rolls hit (the raw rolls span 12.8-99.4,
  uniform-shaped — the shared stream's sample; the rate figure is not pinned beyond
  "lands", which the small n cannot support).
- **Cadence = the game's.** 15 shots with gaps 150-266 frames (minimum 150 ≥ the 135 floor =
  interval 120 + the smallest burst delay 15); the two 266 gaps are the game's
  repositioning pauses, all gaps ≥ the floor, asserted.
- **The sight holds**: zero LOS flickers across the stand (the 120-tick window is meaningful).

**No difference found — there was nothing to fix, so no red-on-old-code test exists.** The pin
is the deliverable: it is the proof the port's sniper deals the game's own 28.0 per round at
15-25 u and fires at the game's own cadence. No draw changes were made (and no draw issue was
found); no src/ file was touched — the branch adds the pin test and the WORKLOG entry only.

## How it was verified

- `walkreal_07000002_live` binary, whole: **4 passed, 1 ignored** (the pin, the Operative walk,
  the Agent walk, the exit walk; the 00 Agent sister stays ignored exactly as on master).
  68.6 s wall.
- The decompile reading was done fresh this session: 0x139f50, 0x1902c8, 0x179740, 0x179d40,
  0x12c5a0, 0x176a28, 0x12ce80, 0x13a940 (the SP caller of ModBulletDamage — no bone scaling).
- The weapon record and the drone stats were dumped from the real level load (difficulty 2) by
  a throwaway probe (not committed); the record values are re-dumped by the pin's log lines on
  every run.
- The full suite did not fit the session window; the branch changes one test file (no src), and
  that test binary ran whole. The other walkreal binaries are untouched by this branch (no
  shared helper changed — `crates/nf-view/tests/common/` is byte-identical to master).

## What's left

- The hit-rate figure over a longer stand (the 3/13 sample is small; the deterministic stream
  makes a longer run a different play, not a bigger sample of the same one). If the driver
  wants a rate pinned harder, the main agent's live trace of the real game's roll stream at the
  crossing would settle it against the same n.
- The pin's Bond dies at the stand's tail (the sniper's 3 rounds + the courtyard's 1.05-pistol
  chasers drain him after ~90 s of standing in the open) — the records complete before death
  and the asserts hold; a variant that survives (armour top-up) would change the play, not the
  pin.

## Live checks for the main agent

1. **Per-round damage in the real game, Agent, no armour**: stand Bond at the stair bottom
   (−36.9, 5.5, −1.5) with armour 0 and watch BL+0x894 (health) under the sniper's fire. Expect
   exactly −28.0 per landed round (× 0.7 difficulty already inside). If the game ever shows a
   health step that is not 28.0 (or 28 − armour when armour stands), the chain above is
   incomplete — please record the step and the armour value at that moment.
2. **The 13.7 round's armour**: in a playtest that reproduces the driver's 13.7 hit, watch
   BL+0x8B0 (armour) at the hit. Expect 14.3 standing before the round and 0 after (the 28-round
   spending it). Any other pre-value breaks the armour-split reading.
3. **Fire cadence**: gdbwatch the sniper's +0xBC4 (next-shot frame) around the crossing — one
   write per shot, 120 frames + 29-59 between shots (149-179), matching the port's gaps.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no cost meter, no token counter is exposed to the agent; the numbers above are absent,
not estimated).
