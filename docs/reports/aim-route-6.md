# aim-route-6 — 0c and 16 walks green test-side; 08's self-blast re-pinned to the rocket's own blast, the attempted gate documented

Branch `s/aim-route-6` from `origin/s/aim-route-5` (691b1c87c), merged `origin/master`
(7370525a4 — the water-byte landing comment; **master's missile countdown stays**). One merge
conflict in walkreal_0700001b_live.rs (comment/declaration hunks both lineages had touched);
resolved on the aim-route-5 side — master's substantive test code (the 550-tick ch-0x09 wait,
the revised objective asserts, the wave defense) was already byte-identical in aim-route-5,
and the merged file equals the aim-route-5 tree. **Test-side only**: no `crates/*/src` file
changed on this branch; the [L] 08 pins and the 0x30000000 fix untouched. Not in the stream
batch.

## 1. 0700000c — the far-sniper drain resolved (the pinned lever, landed)

aim-route-5 §3 pinned the cause: the return-fire stands' converged AIMED rounds miss a guard
that STEPS during the round's flight, and the plain 20 u shooters window never even picked
the d 61-90 snipers (#39 0x5000093, #20 0x5000087, both 0x1e, hp 10). Both levers landed:

- **A new opt-in `sniper_stands` stand in the shared `DamageLog::stand_and_fire`**
  (crates/nf-view/tests/common/damage_log.rs): the level's own `yard_stand_aimed` shape —
  the gun settles (the raise queues only on the aim-PRESSED edge from the idle machine
  [M 0x1a24c8:113-152]), the aim button held, the CURSOR driven to the position that puts the
  EFFECTIVE aim (the view + the eased offsets [M scene.rs free-aim]) on the part, and the tap
  gated on the aim BIT + the effective error 0.01 + the target's root holding still 18 ticks
  (root motion only — the wiggle-step ±0.1 u beats the flight [M Bullet_update 0x12e7b0]).
  The re-pick uses the strict gun rays (the yard idiom). Off = the plain stand everywhere
  else.
- **The shooters window opens to the snipers' stand-off (95, 13)** and the flags ride a
  thread-local in common/walkreal0c.rs (`with_far_sniper_stands`), set at the TOP of
  `walk_spawn_to_the_band_at` — **before the legs**: the snipers open up on the band approach
  INSIDE the prologue (the measured 13461 death sits in it), so the first wiring (after the
  prologue returned) never ran and the walk died byte-identical — caught by probes, moved.

Result: the fights answer the far snipers and ONE landing round kills an hp-10 guard — #20
killed at 13323 (its old kill-shot landed at 13463), the yard stand holds #39 through its
still-gate, and the walk completes the whole mission at **82.6 hp** (20066 + 838 ticks; the
master-green plain walk is untouched — 3/0/1 whole, the agent sister stays `#[ignore]`d as
before).

## 2. 07000016 — the melee-swarm finale resolved

The 14656 death's finishers were NOT the squad clones: they were the pit-room AR guards #15
(skin 0x50000b2, weapon 0x14, hp 14) and #14 (0x50000b1, hp 10) at **d 1.2-1.3, inside the
3.0 always-hit band** [M DroneWeap_DoBulletAccuracy 0x179740:113-115], shooting Bond in the
back while the old clone-only band hold faced away, and the member kill sat inside a
7300-tick fixed-target `shoot_bounded` loop with no threat reaction. Fixes
(crates/nf-view/tests/walkreal_07000016_live.rs):

- The band hold factored into `hold_the_band` and widened to EVERY live hostile (team 1)
  inside 3.0 — a player surrounded turns to the nearest muzzle.
- The member kill runs in bounded slices (12 × `shoot_bounded` 900) with the band hold
  re-checked between slices — the closing muzzle is answered the tick it arrives.

Result: the finale walks out at **6.0 hp** (95 hits for 94.0, the squad-phase audit clean:
50 hits, every delta within the game's chain). The binary is green whole (14/0/2; the d2/d3
sisters stay `#[ignore]`d as before).

## 3. 07000008 — the 00agent self-blast re-pinned; the attempted gate documented

The NF_DMG_TRACE run pinned the 28756 death's "world" bucket (78.0 in 3) to **Bond's own
rockets** — the kill-floor hurt volumes are 37.6 u below the death spot and cannot be
touched (docs/levels/07000008.md's y −24.10 quads); the ledger's −75.0 at tick 28178 is a
kind-44, owner-−1 explosion at (113.6, 14.3, −180.3), **d 2.7** (the falloff's trunc-k 2),
then the Hind's 2.0-damage gun rounds chewed the last 3.0. The round's FIRST ticks fly along
the EFFECTIVE aim (the view + the eased cursor offsets; on d3 the crosshair law saturates the
tracker and launches sit up to 0.23 rad off the view) — 0.6 u of lateral error at 2.7 u, the
jamb's width — so the d3-gated **effective-aim corridor gate** was added (the same three-source
12 u cast along the effective direction, `walkreal_07000008_live.rs`). It prevented the
28178-class detonation but the death moved to **27975** (a −62.5/−15.5 self-blast pair):
the chase's own mid-flight curve from a clear first-12-u corridor remains. **Next lever**: the
forecast gate — extrapolate the machine's flight-time position (`cp' = cp + v̂·(l/0.5)`, the
closing speed) and require the machine_lane_clear checks against BOTH positions — plus a
pin of the round's actual spawn direction (lock-on vs the effective aim) in PCSX2.

## 4. Verification — every quoted binary whole, per-binary counts

| binary | result |
| --- | --- |
| `walkreal_0700000c_live` (whole) | **3 passed / 0 failed / 1 ignored** — was red, now green |
| `walkreal_07000016_live` (whole) | **14 / 0 / 2** — was 13/1/2, now green |
| `walkreal_0700001b_live` (whole) | **17 / 0 / 2** — unchanged green (the two drakes walks included; run with `--test-threads=2` so the two slow walks interleave — the sims are self-contained per test) |
| `walkreal_07000014_live` (whole) | **9 / 0 / 1** — green |
| `walkreal_07000008_live` (whole) | **5 / 1** — the same single red (`…_real_damage_00agent`), now with the §3 evidence |
| `fusebox_geometry_07000014_live` | **1 / 0** |
| `interactables_07000014_live` | **10 / 0** |
| `rocket_glass08_live` | **3 / 0** |
| `rocket_origin_live` | **1 / 0** |
| `rocket_stop_plane08_live` | **4 / 0** |
| `solar_panel_hits_live` | **1 / 0** |

- The six aim binaries: **all green** — the snap stack moved none of them.
- The full 4-crate suite did NOT run (time). No `crates/*/src` file changed on this branch
  (tests + docs only), and every binary whose level the branch touches ran whole, listed
  above. The shared-helper changes (damage_log.rs `sniper_stands`, walkreal0c.rs
  `with_far_sniper_stands`) are opt-in flags no other walk sets; the quoted walkreal binaries
  (08/0c/14/16/1b) are the levels they can affect.

## 5. What's left

1. 08 `…_real_damage_00agent`: the §3 forecast gate + the spawn-direction pin.
2. The 0c agent sister (d2, `#[ignore]`d) and the 16 d2/d3 sisters: their own windows
   (the south-gallery gallery approach for 16-d2; the 0c d2's crossfire cadence).
3. The common-helper opt-ins were exercised only by their own walks; the other levels'
   walks are unchanged by construction (the flags default off) — a full `walkreal_*_live`
   sweep on a longer window would confirm it mechanically.

## Live checks

1. **The 08 rocket's spawn direction** (carried from §3): with the debugger, watch one
   AT-420 round's first 10 ticks at a d3 launch where the view is 0.2 rad off the machine —
   does the round spawn along the VIEW + cursor offsets (the port's effective-aim law) or
   along a LOCK-ON direction toward the machine? The 27975 self-blast died 2-3 u from Bond
   with the effective-aim corridor CLEAR — if the spawn direction is the lock-on, the gate
   must cast along THAT.
2. **The m8 timeline gap (carried from aim-route-5)**: both lineages measure k8 latching
   ~1700 ticks past the P0_LEN model's +5999 branch read with m8 DISARMING — watch the
   kind-0x39 missile at placement (51.78, −12.84, +11.38) in PCSX2 with ch 193/194 held: the
   frame its obj+0xf4 leaves state 2, relative to ch 8's rise and to launch+3600+2399.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
