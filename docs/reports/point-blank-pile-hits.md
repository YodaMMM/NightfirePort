# point-blank-pile-hits — player rounds at ~1 unit into a guard pile

Branch `s/point-blank-pile-hits`, cut from origin/master d07c579e, merged origin/master ca18fd81 at the end (merge d98a277d).

## What the game does for a player round at that range ([M] unless noted)

The game has **no minimum-range gate anywhere** in the player-round path. The chain, every step read in `work/decomp/ACTION.ELF/`:

- **Spawn at the eye, not the muzzle** — `Player_WeaponInitBullet` 0x1a4bf0:152 uses `Player_GetHeadPos`; only for FP-model records (`+0xDC != 0`, `+0x68 & 0x10000`, not the no-crosshair combo) is the start blended 50/50 with the weapon-bone datum [M 0x1a4bf0:233-241] — never fully the muzzle. `Bullet_init` 0x12c5a0:82-101 creates the round at the caller's VECTOR verbatim: no near clip, no push-out, no minimum travel.
- **The first flight tick already covers a 1 u guard** — `Bullet_update` 0x12e7b0:382-384,448-464: state-0 step `(0.5+FRand(0.5))·speed` (P2K speed 5 u/tick [L docs/systems/drone-damage.md:126]), ray mode 0x201 with `HITTEST+0x8C = max(step, 1.0)`.
- **Every overlapping guard is a candidate** — `Collide_Pick` 0x1eb320:313-329 walks every object of every straddled cel (`Collide_StraddleCels` 0x1eac98); `Collide_PickObj` 0x1eae80:93-107 admits a candidate when the segment touches its bounding sphere (centre `obj+0x80`, radius `obj+0x8C` ≈ 1.05 [L drone-damage.md:28]); it rejects only corpses/disabled (`obj+0xF0 & 0x20`), class filters and the exclusion list.
- **Inside hits win** — `Collide_Jointy` 0x1ec180 (via `Intersect_RayBox` 0x1e9008:18-25) returns t = 0 with `HITDATA+0x0C = −1.0` when the segment starts inside a box, so it beats the world stop in the sort.
- **One victim per round, chosen by the sorted list** — `Collide_Update` 0x1ea398:118-158 sorts by `+0x0C`; :216-229 walks to the first non-pass node; corpses are skipped, so a pile soaks rounds front to back.
- **No invulnerability window** — `Drone_BulletHit` 0x13a9f8 gates only on alive + damage > 0; `NDrone2_BulletImpact` 0x1461c8 → `NDrone2_HitDamage` applies part multipliers every hit (the flinch cooldown only gates the animation).

## Repro and pins

`crates/nf-view/tests/pile_hits_07000003_live.rs` (new): four live team-1 guards of 07000003 moved onto one spot, Bond armed with the P2K at 1 u, rounds through the real fire path (`step_bond(fire)`), measured with the game's own accuracy counters (`shots_fired` / `shots_hit_enemy`, the PlrStat pair [M Player_WeaponInitBullet 0x1a4bf0:258; NDrone2_HitDamage 0x145558:57-60, folded at drone.rs:19968-19988]):

- `five_rounds_at_1u_into_the_pile_all_land` — 5/5 land at 1 u (the soak runs front to back through the corpses).
- `five_rounds_at_5u_into_the_pile_all_land` — sister, 5/5 at 5 u.
- `the_walker_shoots_the_pile_at_1u` — the route run's own discipline (the walker's `exposed_aim` + input-driven turn + crosshair/autoaim fire + tap thresholds): every round the walker fires lands.

Harness findings worth keeping (both initially mimicked the symptom): the pile must be **team-1 only** — killing a team-3 servant fails the 07000003 mission and disables firing, so rounds 2+ never fire; and hits must be read from the accuracy row after a fold, because a 5 u round arrives on tick 2 (first step 2.5–5 u) and its impact lands outside the fire tick's window.

## Result on the port

**No gap reproduces on master.** In every state I can construct on 07000003 — held pile, direct aim, the walker's real aim/fire path, 1 u and 5 u — the port lands exactly the game's hits: the round spawns at the eye (scene.rs:5085-5108's midpoint blend matches [M 0x1a4bf0:233-241]), the first segment covers a 1 u guard (`l = step.max(1.0)` drone.rs:13922 = [M 0x12e7b0:455-456]), every pile guard is tested (drone.rs:14438-14471), the nearest solid wins against the world stop (drone.rs:14475-14476 = [M 0x1ea398:216-229]), corpses pass. The one divergence found is strictly more permissive, never a miss: the port's guard pretest is a 2.1 u sphere at the feet origin (drone.rs:14444) where the game uses the posed `obj+0x80`/`obj+0x8C` ≈ 1.05 sphere [M 0x1eae80:93-107].

The 48-round symptom was observed on **origin/s/enemy-stats-3's** full-route run, whose bunched four-guard pile is produced by that branch's placement-stat merge (accuracy/aggression/reaction, DoModeSettingsNEW 0x14c600:81-107 — bisected red after that merge per their commit 5445b2e0). Master does not carry that merge; the enemy-stats-3 diff touches no line of the player-bullet-vs-guard path (`git diff origin/master...origin/s/enemy-stats-3` touches fire_weapon/handle_firing and the drone→Bond damage only), so the symptom cannot be hit from this branch.

### Live checks for the main agent

1. On the enemy-stats-3 tree (the placement-stat guards bunched at (10.4, 6.5, −31.5) in state 0x5C): gdbwatch `Drone+0xAC` (health) of the four pile drones while firing one P2K round at 1 u — expected if the game hits: each round drops health by 3.5×(difficulty/part); if the port-side suspicion were real the health would not move.
2. Same state: fieldprobe `obj+0x80/obj+0x8C` of the four pile guards (the posed world-sphere centre + radius) — to compare against the port's feet-origin 2.1 u pretest sphere.
3. gdbwatch `HITDATA+0x0C` of the round's sorted list at the moment of a point-blank pile shot — expected −1.0 for the inside hit [M 0x1ec180:124-137].

## Runs (quote the totals)

All on the final tree (merge d98a277d), `--test-threads=1`, NF_ISO the onfoot ISO:

| binary | result |
|---|---|
| pile_hits_07000003_live (new) | **3 passed; 0 failed** (2.2 s) |
| walkreal_07000003_live | **4 passed; 0 failed** (129.5 s) |
| nf_game (lib tests) | **635 passed; 0 failed; 5 ignored** (0.9 s) |
| bullet_range_live | 3 passed; 0 failed (2.1 s) |
| impacts_live | 5 passed; 0 failed (6.8 s) |
| fix_flesh_impacts_live | 4 passed; 0 failed (3.0 s) |
| drone_weapon_drop_live | 3 passed; 0 failed (3.2 s) |

The full four-crate suite did not fit this session's window: the sandbox reset at the start cost the setup time, the background `suite.py`/`nohup` runs are killed between tool calls in this sandbox (the committed fallback `tools/test/chunk_suite.py` was prepared for that path but the window closed before a full pass), so the required subset above is what ran; it covers the bullet/impact/drone-damage binaries, walkreal_07000003_live and all of nf-game.

## What's left

- Re-run the observation on a tree that carries the placement-stat merge: if a pile state exists where the port misses what the game hits, the divergence will be in the live-AI pile (flinch/impact anims moving the boxes), not in the flight/pick chain pinned here.
- The broad-phase pretest could be tightened to the game's posed world sphere (`obj+0x80`/`obj+0x8C`) — cosmetic, no behaviour change expected.

## Usage

My own session's token/credit/context counters are **not visible to me** (no usage panel in this environment). The two research subagents' final reports showed, verbatim, in their trailers: "Tokens -> prompt:229514 completion:7876 total:237390" (game-side brief) and "Tokens -> prompt:307049 completion:7557 total:314606" (port-side brief). No cost figure was shown anywhere.
