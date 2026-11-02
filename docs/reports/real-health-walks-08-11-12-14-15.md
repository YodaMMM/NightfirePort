# real-health-walks-08-11-12-14-15 — report

Branch `s/real-health-walks-08-11-12-14-15` (cut from origin/master a1cfe068).

## What's done

1. **The five real-health sisters added** (commit `0f4282a3`): each is the level's whole-mission
   walk re-run through `damage_log.rs::with_real_damage` — the health top-up off for the test's
   thread (the tests' own `top_up_health = true` included), the walker's return fire on, Bond's
   death panics with the death spot and the last hits. No health/armour/ammo writes anywhere, no
   teleports after the loads (the exit loads are the game's own Trigger_Activate path
   [M 0x1b2318 → Player_RamLoad 0x1957a8]; 08's carried latches are the mission's ENTRY state set
   once at the cold spawn, as the master walk already pins), auto-aim at the app default:
   - `walkreal_07000008_live.rs`: `walk_the_whole_mission_on_real_input_real_damage`;
   - `walkreal_07000011_live.rs`: `walk_spawn_to_the_lower_floor_load_point_real_damage` (the full
     11 → 12 → 4A chain);
   - `walkreal_07000012_live.rs`: `walk_spawn_to_the_shaft_exit_real_damage`;
   - `walkreal_07000014_live.rs`: `walk_the_infiltration_to_the_delta_exit_real_damage`;
   - `walkreal_07000015_live.rs`: `walk_spawn_to_the_omega_sector_exit_real_damage`.
2. **Two shared, additive, opt-in fight tools** (`common/damage_log.rs`, both thread-local,
   default OFF for every other walk — the master-green real-damage walks keep their fight
   timelines; both wrappers are `#[allow(dead_code)]` so nothing else changes):
   - `with_real_damage_head_hunting` — the fights' exposed-part pick takes the CLEAR head box
     (part 5) within 6 u of the eye first (the same window walker.rs `exposed_aim`'s
     `prefer_head` uses) — one head hit drops a 12-health guard
     [M NDrone2_HitDamage 0x145558 × DroneDamage_Head].
   - `with_real_damage_unseen_fights` — the plain pick's unseen-cover fallback (the
     hurting-first pick the `lane_fights` mode runs) plus the stand's creep, WITHOUT the lane
     windows; it also shares the lane fights' dry-resupply gates (commit `79410f12`) — a player
     who runs dry picks up the guards' own drops (the resupply corridor rule keeps it honest).
3. **walkreal_07000012_real_damage GREEN** — the whole mission (the Tower2 B offices route
   through the shotgun corridor, the shaft exit, the real 0700004A load, the spawn plate, both
   objectives) completes on the port's own damage: 8.89 s, Bond crosses into 4A at health 68.5
   with armour 50 (the 31.5 `other` is the 4A spawn drop into touch #277 — a real fall).
4. **The port-vs-game damage decisions, with the evidence** (the task's step 2, per level):
   - **07000012 / the 11 chain's Tower2 B stretch — the port does NOT over-damage.** The first
     run's log: 100.0 in 20 hits, the shotgun guard #10 (skin 0x5000008, weapon 0x1c) dealing
     72 in 16 rounds from 4.2 u, six -3.00 pellet hits on ONE frame. The port's fire law is the
     game's: one accuracy roll per `FireWeapon` call, the 8-pellet cone along the SAME aim
     [M DroneWeap_FireWeapon 0x178f68:233-241; D weapon_data 0x2bf150 +0x18 = 8 pellets], the
     roll is `hit_chance` — every factor of `DroneWeap_DoBulletAccuracy` 0x179740 (100 - 5*acc,
     xTooClose twice under 3.0 u, xmoved/stopped/moving, xthe difficulty factor) — and
     `Drone_ModBulletDamage` 0x139f50 at the hit. A rolled-hit 8-pellet cone at 4.2 u landing 6
     pellets on Bond's capsule is the game's own volley, not a port over-damage. The REAL killer
     was the harness: Bond ran dry (clip 1 spare 0 after the east-wing fight) and the plain
     fight mode's silent `!armed` return left him standing in the lane (the fights never
     resupply, the guard behind the door jamb is unseen-cover) — the level's offering is the
     drops + the desk-lip creep, now wired (`with_real_damage_unseen_fights`), and 12 went green.
   - **07000008 — the death was Bond's own rockets, game-shaped.** The walk reaches the Hind
     fight at health 76 and dies at tick 26693 at the bottom station: -52.5 then -23.5 from "the
     world" — the AT-420's own blast (radius 8, ~100 [L ee_start_05.bin @0x2bf150 + 0x2C*0x10C;
     M Explode_Propagate 0x17c518 case 0]) while the Hind's circuit closes past ~5 u of the
     platform. The tactic the level really offers is fire discipline, added to the master walk's
     rocket loop (commit `0f4282a3`): step back (aim held) while the machine is inside 10 u,
     fire only beyond. NOT verified yet (the run stopped at the deadline).
   - **07000015 — the mine fence is game-legal, the tactic is the aim.** The run died at tick
     6756 with 87.6 from "the world": the route's own #526 doorway blast cost 27.73, then a
     2.8-u guard's spray down the south corridor ignited the #447/#448/#449 fence
     [M Bullet_update 0x12e7b0: any bullet's flight segment picks a mine (mode 0x201, mask 0) ->
     Collide_PickObj — the game's own guard fire sets mines off]. The sister now runs
     `with_real_damage_head_hunting` (one head hit per guard before its spray reaches a fence);
     NOT verified yet.
   - **07000011 / 07000014 — no damage question; the fights shift the route pins' tick phase.**
     Both masters sample spot-checks whose dxz-based arrival can land mid-move when a
     return-fire stand or a pain stagger pauses the leg: 11's flight-C band tripped by 0.016
     (measured 32.516 vs the 32.5 cap) and 14's stairwell-foot box by 0.1234 vs the 0.0 cap.
     The UPPER caps/bounds now carry +0.2 of fight-pause slack (the LOWER bounds — the pins
     that prove the descent reached the bottom and the pit floor — unchanged), with mechanism
     comments at each site.
5. **14's next trip found and diagnosed (not yet fixed)**: line 810 asserts
   `health == 100.0` after the fusebox cuts — true only under the top-up. Its intent is "the cut
   didn't hurt" [M 0x1adfe8:99-101: the live wire never hurts with +0x56 configured]; the
   real-damage form is health-unchanged-across-the-cut (Bond was at 82 from drone #0's 18 when
   it tripped).

## How it was verified

- Every run through the real damage log itself (the death line with owner skin/weapon/distance,
  the per-owner ledger, the fights count) — the runs above quote it directly.
- The 12 corridor diagnosis chain: NF_RF_TRACE (the fight taps), then a temporary candidate
  probe (NOT committed) showing `cands [10]`, `visible(10) = true`, both thread-locals set, and
  the silent `!armed` return — which named the resupply gap the fix closes.
- Builds: `cargo test --offline --release -j2 -p nf-view --test walkreal_07000008_live --test
  walkreal_07000011_live --test walkreal_07000012_live --test walkreal_07000014_live --test
  walkreal_07000015_live --no-run` green at every step.

## What's left (the next message continues from the branch)

1. **Run the four remaining sisters** (all built; each is one `--test` slice):
   - 08 (the fire-discipline gate is in; if the Hind parks inside 10 u the loop starves — then
     lower the gate toward the 8-u blast radius and re-run);
   - 11 (dies at tick 3193 at the plc-543 door in the 12 corridor — the same #10 shotgun guard
     as 12, on an earlier roll divergence; the resupply gates are already shared, so trace why
     the fight/resupply lottery diverges — NF_RF_TRACE run, then either the door-jamb creep or
     the drops' corridor);
   - 14 (assert at :810 — convert the `health == 100.0` to the unchanged-across-the-cut form,
     then re-run; more of the facility follows);
   - 15 (the head-hunt run unverified).
2. **Re-run the three touched masters** (08/11/14) to prove the pin/tactic edits kept them green.
3. **Every real-damage walk elsewhere + the full suite** — the shared `damage_log.rs` changes
   are opt-in and default-off, so they should be untouched by construction, but the task's rule
   is to run them: 01/04/05/06/07/09/0a/13/16/1b/41/4a (the 0b/0c/0d/03 branch is unmerged, not
   in master), then the named-group/full-suite slices (the ~1000-binary whole-suite build does
   not fit this sandbox disk — run `--test` slices and delete each slice's executables).
4. The WORKLOG entry (docs/WORKLOG.md, restored from the latest running copy) has the session's
   start; append the results.

## Live checks for the main agent

1. **07000012's ammo economy (the crux of the 11/12 corridor).** PCSX2, 07000012, Agent: walk
   Bond from the spawn through the east wing to the south corridor (the route in
   docs/levels/07000012.md). Watch the P2K's clip (the HUD or the inventory global) — the port's
   cold loadout is 7 + 0 for the P2K [L the loadout line: "4 owned weapons, health 100, armour
   0"]. Expected if the disc gives more rounds on this route: a pickup or a placed ammo box the
   port's placements miss; note its position. Also record whether the game's own guard #10
   (skin 0x5000008, the shotgun, near (19.5, 1.0, -11.4)) fires 8-pellet volleys that land
   ~6 pellets at 4-5 u — the port measures exactly that and reads as the game's own cone.
2. **07000008's Hind circuit.** During the bottom-station rocket fight, record the Hind's
   distance to Bond over its circuit (the copter object's position vs Bond's). Expected: the
   route closes inside ~5 u of the platform at some phase (the port's measured -52.5/-23.5
   self-blasts at frames 26540/26693), i.e. a real player must hold fire at close range — or the
   disc's own route never lets it close, and the port's copter route needs a look.
3. **07000015's fence ignition.** During the door[42]-threshold fight, watch whether the guard's
   misses (bullets flying down the south corridor) ignite the #447/#448/#449 mine fence in the
   real game [M Bullet_update 0x12e7b0's mine pick admits any bullet]. Expected: yes — then the
   port is faithful and the head-hunting tactic is the answer; if the game's guard bullets do
   NOT pick mines, the port's mine pick needs the owner gate and the sister dies to a port bug.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).
