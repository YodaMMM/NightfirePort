# fx-muzzle-repin — landing s/fx-muzzle-and-trail on current master

Branch `s/fx-muzzle-repin` = master `811cb3e39` + the `s/fx-muzzle-and-trail` merge (6d6d97454) + the
re-pin commits. The merge is the point: master had moved (`audit-sp-panes`, the lens-flare work), the
branch was cut from `17a784980`, and the rebase shifted the shared random stream. Per the live PCSX2
ruling (0x178f68 entered 40/40 guard shots, 0x179248/0x179268 39/39, run ended on the 40th) the three
jitter draws are right and stay — every fix below re-derives expectations on the corrected stream and
removes or reorders **no draw**.

## The merge

- `gas.rs` content conflict: master's kind-9/10 emitter arms (fx-gas-and-blasts row 5) and the
  branch's kind-0xb big-puff arm are different match arms of `0x1ab2d0` — both kept, the arm-10
  closing brace restored (the auto-merge had left it in the shared region).
- `gas.rs` literals: the branch's three new `Gas` literals (`init_gt_shot_smoke`, `init_gunimp_smoke`,
  `init_trail_big_puff`) got master's `hidden/born/child/rot` fields on their own lines (lesson 4/9);
  `nf-game --lib` compiles and its one merged test call took `update(0)` (the frame arg master added).
- `docs/WORKLOG.md` add/add: master strips the worker WORKLOG (process file); the merge keeps the
  branch's entries and appends this task's.
- `drone.rs` (the stale-comment ask): greps for `never fire|unconsumed|fake-seed|no render sprite|`
  `0x1792|0x40 sprite|sprite` find the fire_weapon block already rewritten by the branch — the
  fix-guard-weapons3 history comment (drone.rs:11329-11333) now states the object is created and the
  three draws made on every live shot (`control_create_object` is a plain `Mem_Malloc` [M
  0x1344e0]), and `shot_flashes` (drone.rs:1095-1101) documents the recorded jitters. No remaining
  "never fire" claim about these draws is in drone.rs; nothing to correct.

## Fixed on the corrected stream (each verified green after the fix)

- `enemy_accuracy_live` (nf-view): the pin matrix re-measured — 0x12 diff-1 cells 11/10/17 (5/10/20 u;
  were 15/11/9), diff-2 d20 17 (was 10), 0x0e diff-1 3 (was 0); all still against the decomp's own
  chance (`rolls_match` holds, the per-hit cost law unchanged at `weapon_data+0x0C × +0x100 × Plr_DMod`).
  The miss-clip bound `hits ≤ rolled + 1` → `+2`: a rolled miss still flies its wobble bullet [M
  0x178f68:233-241 — the FireWeapon loop runs whatever the roll] and the realization clipped Bond at
  20 u; the doc comment says exactly that. **5 passed / 0 failed** after re-run (413 s).
- `gun_turrets_live`: the wall-gun burst golden re-measured — damage still 24000 flat every round
  (the damage law did not move), draw positions +12 and 460-draw gaps where a level consumer now
  spends its game draws between turret rounds. **2 passed / 0 failed**.
- `upward_shot_0700000c_live`: the hip-pull pin re-derived — the pre-fix "3 dry clips" was a
  realization of the missing-draw stream; on the corrected stream the first tap's round lands (600
  dmg into the 10-hp emplacement). The cone law [M 0x12c5a0:139-170] is untouched; the aimed-sister
  test still contrasts. Live re-check requested below. **2 passed / 0 failed**.
- `smg_close_range_live`: the pistol legs tap up to 3 rounds (a player re-pulls on a flinch-dodge
  miss), and an ally-killed target is re-picked (on the corrected stream a neighbour's 0x1c blast ate
  d#6 — 3 pellet impacts, 13.5 dmg — before the SMG rounds landed; a player re-picks). All original
  asserts kept for the my-weapon path. **1 passed / 0 failed**.
- `fix_guard_weapons_live`: `collect_shots` trips each candidate's OWN switch channel before parking
  (the game's Enable path [M NDrone2_DSTATE_WaitSwitch → NDrone2_Enable]; the same affordance
  enemy_accuracy's `measure()` uses). On the corrected stream the 0x0e fight no longer leaves
  07000001's 0x12 guards sight-capable (they sit WaitSwitch-disabled, vision-off [M 0x176a28: the
  0x10000 bit] — no sight pipeline, `seen 0` forever, 8 warmup re-parks × 7 candidates all cold).
  **3 passed / 0 failed** (the 0x0e min-gap-60 clamp and the 0x12 ≤6 burst-cut asserts all held).
- Branch-touched nf-game binaries on the merged tree: `ai_live` **37/0** (the re-seed 9→16 holds),
  `guard_drops_flashbang_live` **4/0**, `grenade_rattle_live` **4/0**, `blast_smoke_live` **6/0**,
  `nf-game --lib` **677 passed / 0 failed**. `trail_big_puff_live` (the branch's new pin) **1/0**.

## Not done — the next message should start here

The window closed mid-fix. Still red on the merged tree (all stream-shift re-derivations, no draw
touched):

- `walkreal_07000001_live`: both real-damage walks DIE — the contact walk to chip damage at 14826
  (drone #17, 0x12, d 10.8), the 00agent walk to a shotgun blast at 4082 (d 2.8). The guards land
  more now; the walks need to fight back or route around (walker.rs is shared — opt-in flag only,
  lesson 7, then run EVERY walkreal). 6 of its 8 tests pass; `walkreal_07000002_live` is **3/0**.
- `walkreal_07000003_live`: both walks stick at "the east lane" (6000-tick cap, d≈6, segment clear)
  — the guards fight/move differently along the lane and block the nav.
- `walkreal_07000011/14/16`: not run (the window).
- `fix_npc_doors_live`: `the_escort_opens_the_interior_doors_on_his_route` — the escort settled at
  (4.6,6.3,6.3), never led past the shoji doors (his reactions shifted).
- `fix_pickups_trace_live`: `with_dev_god_..._the_spare_matches_the_trace` — drop 5 (0xe) is taken
  where the trace expects it untakeable (the drop sequence shifted).
- `start_corridor_guard_07000011_live`: `idle_bond_is_approached_and_shot_at_every_selectable_
  difficulty` — idle Bond SURVIVES d2 within 1500 ticks now (the guards' approach/aim realization
  moved); 7 of 8 pass.
- Per lesson 10 (drone.rs touched): `npc_doors_live`, `fix_guard_misc_live` unrun; the full suite
  (`cargo test … -p nf-game -p nf-formats -p nf-audio -p nf-view`) did not fit the window.

## Live checks for the main agent

1. **The upward-shot hip realization** [M→L]: PCSX2 0700000c, stand at the yard ladder base
   (11.6, 1.2, −186.6), hip-fire the sniper 0x1e at the deck sentry (#18, 36.40, 12.06, −208.79) —
   does the real game's hip pull land a round on the corrected stream (the port now kills on the
   first tap), or stay dry? The old [L 0x2c10b8] dry pin was measured on the pre-fix port stream.
2. **Walkreal 01/03 re-derivation** needs no new live work if the driver accepts the re-pin flow;
   otherwise gdbwatch the two 07000001 contact guards' shot count over the walk window.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures.
