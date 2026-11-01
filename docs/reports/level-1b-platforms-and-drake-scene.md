# level-1b-platforms-and-drake-scene — report

Branch `s/level-1b-platforms-and-drake-scene` (from origin/master). WORKLOG:
`docs/WORKLOG.md`, entry `level-1b-platforms-and-drake-scene`.

## Part 1 — the "breakable platforms": what the disc says

The station carries **no 0x20 breakable placements at all** [D dump of 0700001b.bin: only 0xE1
and the scripted kinds]. Two candidates hold the user's symptom:

1. **The 0xE1 destroyables** — 6 placements [D map 26: PL#1 health param 3 = 10000; PL#224/260/
   263/276/278 health 0]. The game's death rule is **bullet-only**: `Destroy_CollisionHandler`
   0x12bd48 walks the contact list and subtracts **only class-0x05 (bullet)** contacts' damage
   from tag+0x04; `Destroy_Update` 0x12c4f8's second arm is the watched-channel smash. There is
   **no player-contact break path** [M both decompiles, full walk].
2. **The 16 solar panels** (SP#41-56, script 0x06000832, hit points param 7 = 20 [D]) — held
   until the damage accumulates; the game's unhold rule is `hp <= damage_sum` [M SP_Update
   0x1dbda8:283-287 `*(float*)(iVar2+0x14) <= fVar12` → SP_UnPause], so **one 20-damage round
   breaks a panel by design** — the start weapon 0x33's damage is exactly 20 [D weapon_data].

**The port matches both rules** (`props.rs bullet_hit`: the mask filter [M 0x1ece30:26-33], the
health drain, the `< 0` kill; `nis.rs` the damage tail with the `<=` boundary). Verified by
probe: an idle/drifting Bond breaks nothing in 900 ticks — the port has no body-contact break
either. Pinned in `crates/nf-view/tests/platforms_drake_scene_1b.rs::an_idle_bond_breaks_no_destroyable`
(the no-touch sister + the disc health values).

**What's left**: the user's exact repro — WHICH platforms (the missile-deck four? the hub pair?
the pit panels?) and WHAT "touch" means (a zero-g bump? a graze by a shot?). The port's
data-level rules match the decompiles, so if the symptom is real it lives in one of:
- the port's blast path (`Explode_Propagate` case 0x21 — the destroyables take blasts when
  `mask & 0x20 == 0` [M 0x17c518]) reaching farther than the game's;
- the panel hit VOLUME: the port's bullet pass tests `segment_hits_sphere` against the SP's
  glist sphere (held_hit_targets) — the game's Collide pass tests the object's own hit geometry;
  a bigger volume makes stray shots count;
- or the zero-g body contact registering somewhere this session did not reproduce.
The main agent's live checks (below) decide.

## Part 2 — Drake missing from the cut-scene

**Which scene**: the level's boot cut-scene is the PACK script **0x0600085c** (0780001b.bin,
2000 ticks, 12 streams [D]). It binds TWO anim actors:
- stream 1: `ANIM(0x0500008f, 0x060008a5)` — **Drake** (the station centrepiece skin — the same
  skin drone #140, the hub's centrepiece guard, carries [D 0700001b.md §4]);
- stream 2: `ANIM(0x0500007c, 0x060008a6)` [D].
Both skins live in 0700001b.bin on **skeleton 0** [D skin dump], the anim scripts are kind-5
entries of the level bin [D], and both actors' meshes resolve [D 0x0200087d / 0x02000854].

**The port's data path is complete — pinned end-to-end**
(`platforms_drake_scene_1b.rs::the_arrival_scene_binds_drake_and_the_pose_resolves`):
- the Scene holds BOTH cast rows (chars with `script Some((0x0600085c, 1/2))`, skins 0x8f/0x7c);
- the sim ticks both `Obj::Anim` streams (frame > 100 of the 2001-tick anim by tick 401);
- both poses resolve on skeleton 0 — 73 bones, Drake's root at **(2.13, −0.03, 0.44)** (the hub
  centre, beside the centrepiece guard's spot).

**What's left**: the data says Drake SHOULD draw — the failure must be in the draw layer
(cel/frustum culling of the script actors at the origin-matrix instances, the GPU palette path,
or a pose-to-render handoff). This sandbox has no GPU: the main agent renders. Render spots
below.

## Live checks for the main agent

1. **Render (Drake)**: `--bond 12.0,-1.9,-1.0,0.2,0.0` during the arrival cut-scene window
   (the first ~2000 ticks / before the controls free): Drake (skin 0x0500008f) should stand at
   the hub centre (≈(2.1, 0.4, 0.4)) at his console. Also `--bond 2.1,1.0,6.0,3.14,0.0` looking
   at his spot. If Bond's actor (0x0500007c) draws and Drake does not, the render difference
   between the two cast rows is the bug; if NEITHER draws, the script-actor draw slot is the bug.
2. **Repro (platforms)**: fresh boot 0700001b, difficulty 3, float Bond (substate 8) into each
   candidate — the missile-deck platforms at (−44.1, −19.3)/(−44.9, −19.8)/(−45.7, −19.8), the
   hub pair at (−12.95, 0, ±0.14), and a pit solar panel — with NO trigger pressed. Record any
   shatter: which object hash breaks on which contact. gdbwatch the destroyed object's
   `tag+0x04` (health) writer if it fires — the writing function names the arm
   (`Destroy_CollisionHandler` 0x12bd48 vs an unexpected contact source).
3. **Panel volume**: shoot ONE round past a solar panel's EDGE (within ~2 units, not at it) and
   watch the panel SP's `SP+0x18` (the damage accumulator). If it moves on a near-miss, the
   port's sphere-volume hit definition is the divergence (the game's Collide pass tests the
   object's geometry, the port tests the glist sphere).

## Runs (this branch, before the finish push)

| binary | result |
|---|---|
| census_0700001b | 0 passed; 2 ignored (the opt-in dumps) |
| interactables_0700001b_live | **7 passed; 0 failed** |
| interactables_0700001b_probe | **1 passed; 0 failed** |
| interactables_0700001b_console_probe | **5 passed; 0 failed** |
| walk_0700001b_live | **1 passed; 0 failed** |
| walkreal_0700001b_live | **10 passed; 0 failed; 1 ignored** (the pre-existing ignored walk) |
| zerog_0700001b_live | **1 passed; 0 failed** |
| platforms_drake_scene_1b (this task) | **2 passed; 0 failed** |
| nf-game (lib) | **987 passed; 0 failed** |

The fast runner (`tools/test/suite.py`) was started detached twice (04:51 and 05:19 UTC); the
first run died with the session break before producing a total line, the second is building at
report time — no FAILED/TIMEOUT lines yet. The next message should quote its total line on this
tree.

## Usage

not visible to me
