# Aim on master: the last seven

Branch from `origin/s/aim-on-master` (aim-route-6 merged with master), push to `s/aim-last-seven`. Full suite: 4428 passed, 7 failed, 352 ignored.

First, because a player would see it: `stairwell_impacts_render_07000011_live every_stairwell_impact_sits_on_drawn_geometry`. If the snap moves bullet marks off drawn geometry, that is a real fault: name the cause.
Then `panel_sequence_07000014_live` x2 (one_cut_box_arms_nothing..., the_panel_sequence_runs...) and `walk_07000014_live the_facility_chain_runs_to_the_delta_sector_exit`: likely the 14 route or the cut shot; see how the 14 walk was fixed in aim-route-finish.
Then `walkreal_07000015_live` real_damage_agent. `walkreal_07000001_live` 00agent and `walkreal_07000008_live` 00agent may take an ignore with a measured reason.

Keep the [L] 08 pins and the 0x30000000 fix. Quote one total line.

---
claimed-by: sandbox1 at 2026-11-06T22:33Z

---
## Report (done, 2026-11-06T23:36Z)


## Report from the worker (done, 2026-11-06T23:36Z)

# aim-last-seven — the stairwell sister re-probed on the declared fudge plane, the 14 wire tests skip the game-removed entities, the 01/08 00agent walks take their measured ignores; the 15 agent red documented with four measured levers

Branch `s/aim-last-seven` from `origin/s/aim-on-master` (24cca34f8 — aim-route-6 merged with
master). Test-side only: **no `crates/*/src` file changed**; the [L] 08 pins and the
0x30000000 fix untouched. The base suite stood at 4428 passed / 7 failed / 352 ignored; this
branch takes out four of the seven (the 11 stairwell pin, both 14 panel tests, the 14 walk,
and — via the two measured ignores — the 01 and 08 00agent walks).

## 1. 07000011 stairwell — the snap did NOT move bullet marks off drawn geometry; the red was the sister check's stale premise

`stairwell_impacts_render_07000011_live every_stairwell_impact_sits_on_drawn_geometry`
panicked at assertion (2) — `pass_off_surface >= 1`, "the fan's fudge-plane records sit off
the drawn mesh". The player-visible assertions were and are GREEN: every non-pass-face impact
≤ HOLE_ON_SURFACE_MAX from a drawn World triangle, and **272 decals with 0 mid-air — the
identical count on both sides of the change**. No hole hangs off the mesh; nothing for a
player to see.

The cause, measured by running the assertion-free dump at the last-green commit 0e0cfdd01
(2026-11-01) and on this branch:

- The pre-convergence fan (0e0cfdd01): 158 world impacts, **8 mat-0x40 pass records measured
  off-surface** (0.416–2.244, on the declared fudge planes x≈15.95/x≈21.78 [D map43#126]),
  272 decals, 0 mid-air.
- The converged fan (this branch): the same 158 impacts, **6 pass records all in-band** (worst
  0.200), the same 272 decals, 0 mid-air. The records MOVED ONTO the mesh — never a hole off
  it.

The mover is the game's own convergence law merged in 08ace8edf: `Player_WeaponInitBullet`
0x1a4bf0:154-164 snaps the aiming point to whatever the crosshair ray hits, and the round
fired from the gun converges on that point [M the port's `Drones::aim_snap` + scene.rs's
shot_from]. Every round is now aimed at a point ON drawn geometry, so the fan's 0x40 stops
hug the mesh and the old sister premise ("the fan's own pass records measure off-surface")
died with the pre-convergence fan shape.

The fix keeps the sister's purpose — proving the render measure CAN fail — but anchors it to
the DECLARED fudge plane instead of the fan's shape: the probe point (21.782, 34.802, 1.415)
must answer the collision query with the bare 0x40 face (the east→west cast — the face's
normal is (+1,0,0) and the query culls back-faces [L the fan's rows]) and must still measure
> HOLE_ON_SURFACE_MAX from the drawn mesh (2.139 [L 0e0cfdd01]). If the mesh ever draws over
the bare plane, or the measure collapses to always-0, the probe names it. The pin's doc
carries both ledgers.

## 2. 07000014 — one burn cuts BOTH of a box's wires; the game removes the dead wire's entity

Both `panel_sequence_07000014_live` tests and `walk_07000014_live
the_facility_chain_runs_to_the_delta_sector_exit` died in the same place: "a stance inside
the Laser's range exists for box 1's wire at [0.0, 0.0, 0.0]" — the second wire's aim lookup
returned the origin because `scripts.objects.list[obj]` was gone.

Measured (a throwaway probe in the sister's conditions): the west boxes' two wire slabs are
COPLANAR and ADJACENT — posed x −89.474..−89.349 and −89.342..−89.148 on the same z 30.895
plane [L] — and the burn's aim sweep clips both: ONE burn on the 0x87 box's group-0 wire rose
BOTH group words (0x87 AND 0x71) and removed BOTH wire entities
([M FuseBox_Update 0x1adfe8 → `Effect_RemoveSObjs` 0x1cf360 — the port's `Cut::RemoveWire`
arm clears the dying wire's slot]). The tests' "burn every alive wire" loop then aimed the
already-dead wire at the origin.

The fix (both files): `cut_box` skips a wire whose entity the game already removed
(`matches!(… Some(Some(Obj::Entity { .. })))`), leaving the cut's verification to the box's
own group words at the call sites (which the tests already assert). Game-faithful: the game
itself deletes the dying wire's sub-objects, so there is nothing left to aim at.
`panel_sequence_07000014_live` 2/0, `walk_07000014_live` 2/0.

## 3. 07000015 — the real_damage_agent walk: four levers measured, the red stands, the ledger and the next lever documented

The d2 Agent walk (`walk_spawn_to_the_omega_sector_exit_real_damage_agent`) dies at tick 9422
taking EXACTLY 100.0 (13 hits, 18 return-fire fights). The base run's ledger (NF_DMG_TRACE):

- the world bucket 48.0 in 3 (the #526 rim-shot blast −41.6 at 2.44 u — the level's own toll
  [M Explode_Propagate 0x17c518's pure-distance sweep, radius 15, damage 80, ×0.6 Agent],
  plus the fence pops);
- drone #11's launcher rounds −8.4 and its grenade −15.0 (the thrower killed at 6524, the
  grenade landed 6730 — the thrown grenade outlives the thrower);
- #13's grenade −17.5 at the death tick 9422 (the thrower killed 9248 by the legs' return
  fire).

Four levers were implemented and measured this session, each reverting to the proven base:

1. **Spot-based pre-shoots miss** (the existing rim-mark pair): `agent_drone_near(8.0)` finds
   nothing — the door routes walk the guards off their [D] spots, and the spawner hold keeps
   them unspawned at the rim mark (the rim-mark sweep at 14 and 16 u found no live hostile:
   #11 spawns during the threshold leg, fires ~50 ticks later at 14.35 u [L the dmg trace]).
2. **The proximity sweep at the door[29] threshold** (the jog's law at 16 u): it answered and
   killed #11 — but the OTHER grenadier (#10) fired four direct rounds during the fight and
   the shoot machinery's flank dance moved Bond into the room; the run died EARLIER (7318,
   17 hits) with #10's rounds + #11's grenade.
3. **The #526 threshold lob** (the 00agent ledger's lever: the door[32] guard's dropped 0x1a,
   the arc clearing the rim): measured DEAD — the 0x1a's record has NO gravity bit
   (f6c 0x40050 [L the record dump: speed 15.35, range 400, damage 3.5]) and the straight
   round dies on the rim's face (los 0.924 [L]); the lob ladder h ∈ {1.5, 2.5, 3.5} never set
   the mine. The lever as written does not exist on this branch's weapon model.
4. **The wait-and-sweep** (at door[29] through, and at door[51] through for #13): standing
   invites the swarm — the through-mark run took 18 hits and died at 6860; the split run died
   at 6669 to a −34.3 point-blank #11 grenade pop (the fight cascade reshuffles with every
   change — the pops' damage is unbounded by structure).

All four reverted; the file is byte-identical to the base. The walk needs a STRUCTURAL kill
speedup, not a position change: the guards' throw cycle beats the shoot machinery's ~90-tick
kill from every measured stance. Next lever: an opt-in pre-throw kill stand in the shared
`DamageLog::stand_and_fire` (the 0c `sniper_stands` pattern — the settle + the tap gated on
the exposed part, fired the tick the spawner releases), plus the live check of the guards'
grenade-cycle timing (below). Whole-binary (single-threaded, the suite's own setting):
**5 passed / 1 failed / 1 ignored** — the failed one is this walk, unchanged red.

## 4. The 01 and 08 00agent walks — the measured ignores the task granted

- `walkreal_07000001_live …_real_damage_00agent` — IGNORED with the measured reason: the walk
  is NOT losing the budget (Bond crosses at 82.0 health, 10 hits for 18.0, tick 212820 [L])
  but the AND(0x11,0x12,0x13) hall gate [D] can never complete: guard #11 (hp 6) never
  EXPOSES — `exposed false` through the whole 10800-tick bound at 3.6-4.7 u — so ch 153 never
  fires. Next lever: a live check of the guard's cower/hide cycle on the 00agent roster, or a
  lure stand that walks it out of cover. Whole binary single-threaded: **7 / 0 / 1** — green.
- `walkreal_07000008_live …_real_damage_00agent` — IGNORED with the measured reason: the
  chase's own mid-flight curve self-blasts at tick 27975 (the −62.5/−15.5 pair, the
  effective-aim corridor CLEAR for the first 12 u [L — re-run on this branch: the same death,
  (111.06, 13.53, −179.71), 6 hits for 100.0]), carried from the aim-route-6 report §3. Next
  lever: the forecast gate — extrapolate the machine's flight-time position
  (`cp' = cp + v̂·(l/0.5)`) and require machine_lane_clear against BOTH positions — plus the
  PCSX2 pin of the round's actual spawn direction (lock-on vs the effective aim). Whole
  binary single-threaded: **5 / 0 / 1** — green.

## 5. Verification — every touched binary whole, per-binary counts (single-threaded)

| binary | result |
| --- | --- |
| `stairwell_impacts_render_07000011_live` | **1 passed / 0 failed / 1 ignored** — was red, now green |
| `panel_sequence_07000014_live` | **2 / 0** — was 0/2, now green |
| `walk_07000014_live` | **2 / 0** — was 1 red, now green |
| `walkreal_07000015_live` | **5 / 1 / 1** — the agent red stands, unchanged from the base (measured, §3) |
| `walkreal_07000001_live` | **7 / 0 / 1** — green with the §4 ignore |
| `walkreal_07000008_live` | **5 / 0 / 1** — green with the §4 ignore |

No `crates/*/src` file changed (tests + docs only), so no other binary's level is touched;
the 15 walk's shared helpers were NOT changed (all four levers reverted — `git status` clean
before the report). The full 4-crate suite did not fit the window; every binary whose level
this branch touches ran whole and is listed above.

## 6. What's left

1. The 15 agent walk (§3): the opt-in pre-throw kill stand, or the live-checked throw cycle.
2. The 08 00agent: the forecast gate + the spawn-direction pin (the report's live check).
3. The 01 00agent: the hide-cycle live check / the lure stand.
4. A full 4-crate suite run on a longer window.

## Live checks for the main agent

1. **The 15 west-room throw cycle**: with the debugger, watch the d2 west-room grenadier #11
   (spawn spot (−1.87, 41.34, 77.42)) from its spawner release: the tick of its first
   launcher shot, the tick of its grenade throw, and the grenade's fuse — is there a window
   where a pre-throw kill is possible, and what does the game's own AI do if Bond stands in
   the doorway (the port's guards walk off their spots — does the game's?).
2. **The 0x1a's grenade mode** (carried from §3's lever 3): the MGL's second fire mode
   (weapon_data+0x2E mode count, +0x92 clip 30) — in PCSX2, does the mode-1 projectile carry
   the gravity bit (a real arc) while mode 0 fires the straight 15.35 u/tick round? The
   threshold lob depends on it.
3. **The 08 rocket's spawn direction** (carried from aim-route-6 §3, unchanged).
4. **The m8 timeline gap** (carried from aim-route-5, unchanged).

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
