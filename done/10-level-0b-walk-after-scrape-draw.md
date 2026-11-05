# level-0b-walk-after-scrape-draw — land s/fx-scrape-and-impacts

s/fx-scrape-and-impacts adds the game's own draws (the laser env puff jitter, the wall-puff roll). They are right, but the shifted random stream kills Bond at the end of walkreal_0700000b_live's real_damage_agent and real_damage_00agent at (2.22,58.08,36.54), under fire from drone #4 at 4.5 units. Rebase the branch on master and make those walks survive the way a player would: fight or take cover at that spot rather than standing in the fire. Do not remove or reorder a draw. Impact tests must count bullet holes only, since guards' footprints share the decal pool. Run walkreal_0700000b_live, impacts_live, qclaw_live and impact_remainder_live and report them.

---
claimed-by: sandbox5 at 2026-11-05T07:14Z

---
## Report (done, 2026-11-05T07:51Z)


---
## Report from the worker (done, 2026-11-05T07:52Z)

# level-0b-walk-after-scrape-draw — landing s/fx-scrape-and-impacts

Branch `s/level-0b-walk-after-scrape-draw` = `s/fx-scrape-and-impacts` rebased onto
`origin/master` a89818235 (clean; the scrape branch's four commits ride on top). Task: the
branch's own draws (the laser env puff jitter, the wall-puff roll) are right, but the shifted
random stream killed Bond at the end of `walkreal_0700000b_live`'s `real_damage_agent` and
`real_damage_00agent`; make the walks survive the way a player would, without removing or
reordering a draw, and make the impact tests count bullet holes only.

## 1. The deaths, root-caused

Reproduced first (the rebased branch, before any change): both real-health Agent runs panic with
`[real-damage] Bond died at tick 16047 (frame 16224) at (2.22,58.08,36.54)` — 24 hits for 100.0
over the run, 58.8 of it from **drone #4** (skin 0x5000006, weapon 0x18) standing at
(−0.4, 58.0, 32.9), **4.5 u** away.

`NF_RF_TRACE=1` shows the killer is a STAND, not the sprint: once the deck sprint reached its
old end slot (pp−5.0), the walked take-slot leg's return-fire gate (every 10 ticks,
`walker_b2::walk_to`) picked fight#9 — a dartable deck guard (drone #9, hp 20) whose aim ray the
deck geometry blocks — and `stand_and_fire` held Bond at (2.2, 36.5) for 180+ ticks with
`blocked true` (no dart flies; the fight's no-round window never closes before the 300-tick cap),
while drone #4 chewed 5.6 a hit: health 53.2 → 44.8 → 30.8 → 8.4 → 0 inside the fight. Standing
in the open trading with an unkillable-from-here dart target is not what a player does.

## 2. The fix — the deck sprint runs THROUGH the take (player law, no draw touched)

`crates/nf-view/tests/walkreal_0700000b_live.rs`, the `real` arm of the shared walk body:

- The deck sprint's target is now the **take slot itself** ([pp−0.3, pp−1.0] = (2.23, 41.34) on
  the disc geometry), and the loop also breaks the moment ch 0x3F rises: the take fires **on the
  pass** — the game's touch law is a per-tick distance+LOS check, `Vec_SqDist3D(pickup+0x80,
  player+0x30) < (radius+1.1)²` with `Collide_LineOfSight(..., 0x721)` [M Pickup_Update
  0x18e148:202-220; Pickup_Handler 0x18e650:330-333 writes switch_channels[3]=1; D #280 p3 =
  0x3F] — it needs no stand.
- The walked take-slot `walk_to` (whose return-fire gate produced the blocked stand) runs on the
  warm walks only; the real-health crossing is pure movement end to end — sprint, take on the
  pass, then the existing east jump drive (raw ticks, no return-fire gate). The tranq-fight
  doctrine is untouched everywhere else; **no draw was removed or reordered** (the change is in
  the walk's input script, not in any effect path).
- The stale "green run … 15.4 left" note in the test doc is updated to the new runs.

## 3. The impact tests count bullet holes only

The guards' footprints share the same 200-node decal pool: `Effect_Player` 0x1cec48 takes
`DList(0x3496e0)` nodes for the boot prints with the **EffectInfo+8/+0xc** meshes (the port's
`EffectRecord.foot_decal_4/8`), while bullet holes are the **EffectInfo+4** mesh
(`hashtable_set_sub_object_to_entity_gfx`, `Effect_Bullet` 0x1ce9c8) [M 0x1cec48:84-107]. The
two tests that counted raw pool deltas now key on the +4 column:

- `impacts_live::a_shot_at_a_wall_leaves_a_decal_a_sky_shot_leaves_none`: the wait ends on a
  +4-mesh decal (a footprint cannot stop it early), the count/`sky_new` assert counts +4-mesh
  decals only, the shot's decal is the newest +4-mesh decal, and the draw-slot read uses the
  hole's **own pool node** (`dec.node`) instead of the pool tail.
- `impact_remainder_live::a_wall_shot_draws_the_games_count_and_a_sky_shot_draws_nothing`: the
  hit wait keys on a +4-mesh decal or new debris; the sky sister asserts no +4-mesh decal and no
  debris (a footprint in the pool is not a hole).

Footprints themselves remain unported (drone.rs "footprints not ported"; death.rs "the 0xc decal
arm — the footprint — is not ported"), so these asserts are green before and after — they are
hardened so the footstep-decal work cannot turn them red.

## 4. Verification (every binary whole, --test-threads=1)

| binary | result |
| --- | --- |
| `walkreal_0700000b_live` (nf-view) | **7 passed / 0 failed** — incl. `real_damage` (d1) end health **62.8** (14 hits, 37.2), `real_damage_agent` (d2) **42.0** (16 hits, 58.0), `real_damage_00agent` (d3) **27.5** (16 hits, 72.5 — the 0.8× DroneDamage_Hard scale against Agent's 1.0 [M 0x179740:74-88]); all three mission complete at tick 16076, stuck 0, the one fall is the parachute jump itself |
| `impacts_live` | **5 / 0** |
| `impact_remainder_live` | **3 / 0** |
| `qclaw_live` | **3 / 0** |
| `nf-game` lib | **667 / 0** (5 ignored) — includes the scrape branch's `wall_puffs_spawn_oriented_along_the_normal` |
| `laser_gas_mesh_live`, `blast_smoke_live`, `break_debris_live`, `grenade_live` (nf-game) | **4/0, 2/0, 4/0, 1/0** — the scrape branch's changed-code binaries |
| `rigid_body_wiring_live`, `npc_doors_live`, `fix_guard_misc_live` | **6/0, 6/0 (1 ignored), 2/0** — the wiring pin + the drone.rs-touched door/guard tests |
| `walkreal_07000001_live`, `walkreal_0700000d_live` (samples) | **8/0**, **1/0 (2 ignored)** — other levels unaffected (the scrape branch adds no draws on the ordinary bullet-impact path; the 0b shift came from the keypad burn's laser-impact jitter) |
| `nf-formats` | 136 ok + the **3 allowed `disc_golden` failures** on the slim ISO |
| `nf-audio` | **28 / 0** |

Determinism: no new RNG draws anywhere (the change removes input ticks, it adds none); the
shared bit-exact stream is untouched by this branch.

## 5. What's left

- The nf-view suite **whole** (959 test binaries) did not fit this sandbox's window/disk — the
  changed-code and required binaries above all ran green. No `crates/*/src/` file changed on
  this branch beyond the scrape branch's own (whose changed-code binaries ran green here too).
- The scrape branch's own live checks (the RB buoyancy puff, the laser env puff on an odd/even
  level, the wall-puff orientation render) still stand — this landing adds none.

## Usage

not visible to me
