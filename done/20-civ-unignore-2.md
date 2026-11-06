# Bring back the last two ignored walks

Branch from `origin/master`, push to `s/civ-unignore-2`. walkreal_07000013_live real_damage and walkreal_07000006_live _agent are `#[ignore]`d. docs/reports for civ-repin-finish and civ-unignore hold the measured ledgers (see their branches' history if deleted). Test-side only, remove each ignore as its walk goes green. Do not loosen a pin tagged [L]. Run each binary whole and quote results.

---
claimed-by: sandbox1 at 2026-11-06T01:36Z

---
## Report (done, 2026-11-06T02:28Z)


---
## Report from the worker (done, 2026-11-06T02:29Z)

# civ-unignore-2 — report

Branch `s/civ-unignore-2` (from `origin/master` c4c00c7ec, the civ-unignore merge). 2026-11-06,
written ~02:55 UTC. Task: bring back the last two `#[ignore]`d walks, test-side only, removing
each ignore as its walk goes green:
- `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (d3)
- `walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_agent` (d2)

## Result in one line

**BOTH GREEN, BOTH IGNORES REMOVED.** `walkreal_07000013_live` — 3 passed / 0 failed / 0
ignored; `walkreal_07000006_live` — 6 passed / 0 failed / 1 ignored (the pre-existing 00agent
research ignore, not this task's target). No `crates/*/src/`, no `crates/nf-view/tests/common/`
file touched — every change is inside the two named test files, difficulty-gated so every
passing sister runs its old config. No [L] pin moved; every assert line outside the new code is
byte-identical to the branch I cut from.

## Reproduce first (both baselines re-measured on this master)

Both ran with the `#[ignore]` still on (`--ignored --nocapture`, NF_DMG_TRACE + NF_RF_TRACE):

- **07000013 d3**: death tick 1666 at (3.57,−368.36,25.09), 24 hits for 86.0 — byte-for-byte the
  predecessor's ledger ("86 → 86 (t1666)"). The anatomy was new, though: the walk crossed INTO
  the spawn-guard pocket and died there — #3 (rifle, hp 24) landed three −14 volleys at
  d 2.6-2.7 after the leg walked Bond into him, #7's shotgun finished it; the fights never
  pre-engaged the pocket because the damage-weighted return-fire pick only answers whoever is
  ALREADY hurting Bond, and the earlier unseen-fight creep (fights #0/#4, aim None) had dragged
  Bond backward into the pocket. The "uncapped fight-loop attrition" of the predecessor's
  summary resolves to one shooter: #4, the mezzanine sniper (weapon 0x18 at (14.7,−368.4,45.1),
  d 40-45), who dealt 69.0 of the 86 in the follow-up run while the fights' hip-cone taps
  scattered ~6 u at his range (spread base 100.0 [L ee_start_05.bin @ 0x2c10b8 +0x24]).
- **07000006 d2**: death tick 5799, 39 hits for 100.0, the east-strip pocket (#24 15.4 in 4 +
  #5 13.4 in 7) finishing a 26.8-hp Bond — exactly the predecessor's baseline. Plus the ledger's
  biggest single dealer nobody ever answers: the dock guard #14, posting at the truck d 27-35
  from every early leg — OUTSIDE the fights' plain 20 u shooters window — 25.0 in 13 hits,
  still hp 10 at the death.

## 07000013 real_damage — GREEN, ignore removed

Fix (file-local, the 13 test file only, gated `real_damage && difficulty == 3`):

1. `clear_the_visible(w, label, radius)` — a leg-end sweep: for every alive team-1 hostile in
   `radius` whose boxes the gun-ray set exposes from where Bond stands
   (`DamageLog::exposed_part_gun_rays` — the lane ray set this walk already fights with), stand
   and take him. A player walking a known-guarded lobby shoots the posted guards from the
   doorway before crossing their pocket; the fights' own pick cannot do that (it answers only
   after the first hit).
2. The sweep's stand is `aimed_stand` — the 07000006 file's own idiom copied level-local (the
   merge rules keep shared helpers opt-in; this copy runs only in this file's d3 pre-clears):
   the aim-raise loop [M Player_Weapon 0x1a24c8:113-152; the bit 0x1a2e38:554-561], the
   zero-spread aimed act [M Bullet_init 0x12c5a0:139-170], the cursor-aware convergence
   [M Player_AutoAim 0x1a7c98:50-59], the 18-tick still gate, the strict gun-ray re-pick every
   10 ticks. One adaptation: after every call the sweep forces `dmg.aimed_fights` back to false
   (the stand's early give-ups return before its own restore, and 13's fights must keep the
   plain tap gate — the `precise_fights`-on-chargers lesson).
3. Two sweeps: after the load asserts ("13: the spawn pocket", radius 24) and after "the lobby
   west mid" (radius 24).

Measured run (this tree, the un-ignored test): the spawn-pocket sweep killed #1 and #2; the
west-mid sweep's aimed stand dropped **#4 the mezzanine sniper**; the chain completed —
**end health 8.5 + armour 50.0, 14 hits for 77.5, other 14.0, 13 return-fire fights, 13 walked
out in 11669 ticks, stuck 0, falls 0** (12 walked out in 5050). The predecessor's carries
question (62/72/86 all consumed) is moot — the sniper who ate 69 of the 86 dies at the west-mid
leg end, before the security-room legs; the cover machinery the report ranked as "the missing
machinery" turned out to be aim discipline at range, not position.

## 07000006 agent (d2) — GREEN, ignore removed

Fix (file-local, the 06 test file only; the d1 walks byte-identical — every arm gated
`difficulty >= 2` or `== 2`):

1. **`stand_shooters_down(w, label, radius)`** after the west-lane legs — the aimed-mouth sweep
   with the pick gated on "actually shot Bond in the last 600 frames" (any range to 40 u, rays
   exposed, recent-hitter first). This is the player's own rule and it widens the fights' 20 u
   window to the guards' real reach. **A blind radius clear here is measured WRONG this
   session**: `clear_the_spot(8.0)` at the dismount shot the friendly driver #17 (hp 1,
   (66.1,39.2,28.9), never a shooter) and his death latched the level exit 303 ticks later,
   killing the walk — kept as a warning comment at the site.
2. **The guardroom pre-clears open to Agent** (the three d3-tier `clear_the_spot(13.0)` arms
   become `difficulty >= 2`): the room's own #3 spawn (30.3,41.0,−17.0) was landing
   point-blank during the crossing (3 × −3.85 at d 2.1-2.8 in the baseline).
3. **The ledge-leg aimed stands**: `aimed_stand_the_mouth` after each of the four east-strip
   legs (the route past the strip posts whose rounds reach d 30+ while Bond's muzzle rays read
   blocked from the west anchors — the sweep takes what opens and skips what stays blocked, the
   80-tick unseen give-up). Runs AFTER the descent's mouth tier, so the ch-55 pair is already
   dead — the civ-repin-finish config-4 lesson (pocket stands while the alarm pair lives bait
   its charge) does not apply.

Measured run (this tree): **the whole castle walked in 13031 ticks, end health 3.7, 51 hits for
96.3, 6 return-fire fights, no falls, the objective folded and the party loaded**. The residual
bleed is #5 (still hp 3): his rays never opened from any anchor the route stands at before the
43-step, so he plinked the walk-in lane at 1.92 a hit. The walk is deterministic and green, but
the 3.7 hp margin is the honest next lever: a restock before the ledge legs (the mouth tier's
kills litter the courtyard with drops) so the fights can finish him when a ray finally opens —
the run's own trace shows the answer existed once (the 43-step fight reached him at d ~9 with
the gun dry, "killed false").

## Verification (every binary that compiles the changed files, run whole)

- `walkreal_07000013_live` — **test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured;
  0 filtered out; finished in 37.61 s** (the plain chain, the un-ignored d3 real_damage, the
  d2 agent sister).
- `walkreal_07000006_live` — **test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured;
  0 filtered out; finished in 21.78 s** (the plain walk, the un-ignored d2 agent, the d1
  real_damage, the lamps, the tunnel/gallery, the ground mouth; the 1 ignored is the
  pre-existing 00agent research ignore, untouched by this task).

The full `cargo test` suite did not run in this window (sandbox expires ~03:39 UTC): the two
changed files are test files of exactly these two binaries — no `crates/*/src/`, no
`crates/nf-view/tests/common/` change, no tower2 change — so no other test binary's behaviour
can move; rule 10's grep surface is the two binaries above, both run whole. Determinism
untouched: no RNG draws added (every stand fires through the game's own trigger path), no
wall-clock, no position writes, no teleports, no health/ammo writes; every step is the real
input path.

## What is left

1. **07000006 margin**: the walk ends at 3.7 hp. A restock pass before the ledge legs (the
   mouth tier's kills drop ammo across the landing/strip) would let the fights finish #5 when
   his ray opens at the 43-step (the measured run proved the ray opens there — the fight ran
   dry instead).
2. **07000006 00agent** (the remaining ignore, a different task's target): the d2 tier's
   levers are now all Agent-grade file-local helpers; the 00 ledger (the deck-guard lane) is
   the next ledger to attack with them.
3. **07000013 margin**: 8.5 hp + 50 armour is comfortable, but the "other 14.0" (unrecorded
   loss — a fall somewhere in the 4A/13 chain) is unexplained and worth one NF_DMG_TRACE pass.

## Live checks for the main agent

- **07000006 strip posts (fieldprobe)**: the exposed_part_gun_rays ray set from the east-floor
  anchor (25.3, 42.6, −40.6) to #5 (27.8, 42.0, −35.7) at his walk-in lane positions
  (45.4,42.0,−38.4) → (34.6,42.0,−38.7) → the 43-step fight spot — this session's census read
  rays0 at d 23 and the predecessor's reactive fight read rays open at d ~9.7; pinning WHERE
  the gate opens decides whether the restock lever (above) is enough or the route must bend.
- **07000013 "other 14.0"** (gdbwatch): what writes the 14 health outside `player_hits` across
  the 12→4A→13 chain (the walkreal ledger's `other` column) — a fall in the 4A shaft drop, a
  hurt volume, or a melee hit missing the record.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.
