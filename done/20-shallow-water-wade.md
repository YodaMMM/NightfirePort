# Wading in shallow water

Measured in PCSX2 at 07000002 (-79, 4.68, 28), a shallow 0xE surface: the contact byte reads 0xE, anim+0x60 carries bit 0x10 alongside 8 while Bond stands there, and steps play 0x28. Landing and step sounds were just fixed on master; nothing else was checked.

Trace bit 0x10 of anim+0x60 to its writer and every reader. Check what the game changes while wading against the port: walk speed, ripples from `Env_WaterRipple` on each step, footprint decals on 0xE, the eye height. Use 553 shallow cells on 02 and 24 on 06 by the port's plain ray (06's scan point at (24, -7) turned out to be another surface in the game, so confirm a spot first). Fix each gap with a test on the real level, not hand-built triangles, and prove it red on the old code.

---
claimed-by: sandbox1 at 2026-11-06T12:49Z

---
## Report (done, 2026-11-06T13:23Z)

# shallow-water-wade — wading in shallow water

Branch `s/shallow-water-wade`, from `origin/master` d6245281a ("Landing and steps read the
water").

## Verdict

**The checks are done; no behaviour gap found to fix.** All four wade behaviours the task lists
already match the game on the real 07000002 shallows; two new real-level tests pin them. The
bit-0x10 trace is complete with [M] cites. The one surprise: **07000006 has no reachable wade
spot at all** — the pond's 0xE plane is capped by another surface everywhere — documented with a
live check for the main agent.

## Bit 0x10 of BLData ("anim")+0x60 — writer and every reader

- **Per-tick clears** [M `Player_CollisionHandler` 0x1949a8:27-28]: `+0x60 &= 0xffc7` (bits
  8|0x10|0x20) then `&= 0xff4f` (bits 0x10|0x40|0x80) at handler entry. The drone handler
  mirrors both [M `Drone_CollisionHandler` 0x140968:71-72].
- **Writer** [M 0x1949a8:159]: after the body-query hit chain is processed, the LAST hit whose
  HITDATA+0x51 lacks bit 2 sets `| 0x10`; when that hit's `+0x40` y is above `BLData+0x760`
  (the collision reference point), `| 0x20` as well. So bit 0x10 = "the body capsule had a
  contact this tick", bit 0x20 = "the contact is above the reference point". Standing in the
  shallow 0xE water, the water plane sits inside the capsule shell → the body contact fires →
  bits 8|0x10 together [L PCSX2 07000002 (-79, 4.68, 28)].
- **Reader 1** [M `Player_Collision` 0x194df0:158-178, case 0xc — the swim float]: with bit 0x10
  set, `Vec_Zero(BL+0x50)` and the gravity add is skipped; with it clear, gravity applies. The
  swimmer's surface float is the body contact holding him.
- **Reader 2** [M `NDrone2_DealWithObjHit` 0x13fff0:91-93]: on a drone hit of surface class
  ≥ 0xf with `drone+0xc5 == 0x1d`, a bit-0x10 player negates the vector at `drone+0x480`.
- A register-carried-read sweep over every `+ 0x60)` load in ACTION.ELF found no further bit-0x10
  tests; DRIVING.ELF/SFX.IRX have no `+0x60` flag accesses.
- **Port**: no BL+0x60 flags word is modelled — `grounded` is the direct `walkable || band` rule.
  The swim float is modelled behaviourally (the port's swimmer rests at a constant surface y —
  probe-measured). No walk-substate reader of bit 0x10 exists in the game, so nothing wade-side
  is missing.

## The four wade checks (real 07000002 shallows, (-79, 4.68, 28) [L])

Measured with the new `wade_real_live.rs` (real load, real input; the PCSX2 spot settle, then
paced walking within the water line):

1. **Walk speed** — dry 0.0929/tick vs wading 0.0953/tick (within 2.6%, the shelf's slope): the
   game does not slow the wade [M the walk-speed path reads no water state — `Player_SSWalk`
   0x192310 and `Player_ResetWalkAnim` 0x195a88 read neither BLData+0x50 nor +0x60]. The port
   matches; pinned.
2. **Ripples per step** — the game runs `Env_WaterRipple(hit)` on the footstep-frame hit in the
   water pair [M `Player_DealWithObjHit` 0x194458:92-95; the gate `(mat&0x3f)−0xd < 2` at
   `Env_WaterRipple` 0x17c118:9]. The port calls `env_water_ripple` on the same footstep event
   (scene.rs, the [M] block cites) — the test counts the RIPPLE_MESH gas spawns on wade steps:
   ≥ 1. Matches; pinned.
3. **Footprint decals on 0xE** — the disc's 0xE EffectRecord carries `foot_decal_4 = 0,
   foot_decal_8 = 0` [D]; the game's `Effect_Player` reads the same EffectInfo and spawns none
   when the gfx word is 0 [M 0x1cec48:84-89]. The port spawns no decal on the water plane (every
   decal the pacing produced sits on the dry surface the exiting steps landed on — verified
   per-decal against the surface under it). Matches; pinned.
4. **Eye height** — the game's eye = root + BL+0x908 − BL+0x90c [M `Player_GetHeadPos`
   0x195aa0]; BL+0x908 eases toward 0.7 while `surfaced`, 0.0 otherwise [M `Player_PositionCamera`
   0x18f5d8:114-128] — no wade term. The port's eye stays 0.700 through the wade. Matches; pinned.

Also confirmed at the spot: the feet contact byte reads 0xE [L], the footstep sound for 0xE is
0x28 [D `SurfaceSounds`; L], the landing plays 0x5FB (master's `jump_landing_sound_live`, green),
and the deep-water swim entry matches [M `Player_InWater` 0x195e50 — `pos.y − 0.3 < cel+0x90`;
the port's `enter_water` cites it; probe-observed substate 3 at the pond's deep side].

## 07000006 — no reachable wade spot (the trap, confirmed)

The driver's warning checked out and went further. The pond is `Object_Pondwater` (cel #351) at
(27.5, 38.60, −7.00) [D docs/levels/07000006.md]. A plain ray from high above reads 0xE there
(the water plane is the top surface) — that is what produced the "24 shallow cells on 06". But
the GAME's feet ray runs from the standing root, and a 0xC cap sits ~0.01 ABOVE the 0xE plane:
from the root the cap is hit first everywhere. Settle-scan of **625 grid points** (x 9.5-45.5,
z −25..11) with Bond placed and settled: the settled feet bytes are ONLY 0xA and 0xC — never
0xE. (24, −7) itself: scoped feet ray t 0.633 → hit_y 38.61 = the cap; the plain ray from y60
reads 0xE at 38.602 — the two surfaces 0.008 apart.

## Tests

- `crates/nf-view/tests/wade_real_live.rs` — 2 tests, both green: `the_02_shallows_wade_walks_at_
  the_dry_speed`, `wading_steps_spawn_ripples_no_foot_decals_and_keep_the_eye`. Real level, real
  input from the load (the settle teleport follows master's `landing_in_the_02_shallows`
  precedent at the same spot).
- `crates/nf-view/tests/wade_probe.rs` — assertion-free probe: the 02 wade measurements and the
  06 settle-scan tool (the next message's 06 work starts here).
- No game code changed (`crates/*/src/` untouched this task), so the suite risk is nil; the
  neighbors ran green: `jump_landing_sound_live` 3/0, `walkreal_07000002_live` 3/0.

No fix was needed, so there is no red-proof: every check the task lists already matches. The
tests are the pins that go red if a regression breaks any of the four behaviours.

## What's left

1. **The 06 live check** (below): if the game's contact byte at the pond reads 0xE, the port's
   cel scoping drops the ball (the port's buckets are coarser than the game's cels [S,
   documented]) and the fix is a scoping correction followed by a real-level wade test on 06.
   If it reads 0xC/0xA, 06 simply has no wading and the "24 cells" are all the trap.
2. The [S] anim question (live check 3): if the game switches Bond's walk animation while
   wading, the speed parity I measured could hide a cadence difference.

## Live checks for the main agent

1. **06 pond contact byte**: PCSX2, 07000006, stand Bond on the pond at (27.5, ~39.7, −7)
   (on the cap the port reads 0xC from) and also wading at the pond's interior edge; record
   BLData+0x50 (the contact byte) each tick. Expected if the cel scoping diverges: 0xE where
   the port reads 0xC/0xA.
2. **Bit 0x10/0x20 at the 02 shallows**: at (-79, 4.68, 28) record BLData+0x60 — expect 0x18
   (8|0x10) while standing [L given], and whether the 0x20 variant appears (the hit's +0x40 y
   vs BLData+0x760).
3. **The wade walk animation**: at the same 02 spot, record Bond's anim sequence id while
   walking in the shallows vs on the dry shelf — the game may switch to a wade cycle (the
   speed parity would then hide a cadence change the port lacks).

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
this session.

---
## Report from the worker (done, 2026-11-06T13:25Z)

# shallow-water-wade — wading in shallow water

Branch `s/shallow-water-wade`, from `origin/master` d6245281a ("Landing and steps read the
water").

## Verdict

**The checks are done; no behaviour gap found to fix.** All four wade behaviours the task lists
already match the game on the real 07000002 shallows; two new real-level tests pin them. The
bit-0x10 trace is complete with [M] cites. The one surprise: **07000006 has no reachable wade
spot at all** — the pond's 0xE plane is capped by another surface everywhere — documented with a
live check for the main agent.

## Bit 0x10 of BLData ("anim")+0x60 — writer and every reader

- **Per-tick clears** [M `Player_CollisionHandler` 0x1949a8:27-28]: `+0x60 &= 0xffc7` (bits
  8|0x10|0x20) then `&= 0xff4f` (bits 0x10|0x40|0x80) at handler entry. The drone handler
  mirrors both [M `Drone_CollisionHandler` 0x140968:71-72].
- **Writer** [M 0x1949a8:159]: after the body-query hit chain is processed, the LAST hit whose
  HITDATA+0x51 lacks bit 2 sets `| 0x10`; when that hit's `+0x40` y is above `BLData+0x760`
  (the collision reference point), `| 0x20` as well. So bit 0x10 = "the body capsule had a
  contact this tick", bit 0x20 = "the contact is above the reference point". Standing in the
  shallow 0xE water, the water plane sits inside the capsule shell → the body contact fires →
  bits 8|0x10 together [L PCSX2 07000002 (-79, 4.68, 28)].
- **Reader 1** [M `Player_Collision` 0x194df0:158-178, case 0xc — the swim float]: with bit 0x10
  set, `Vec_Zero(BL+0x50)` and the gravity add is skipped; with it clear, gravity applies. The
  swimmer's surface float is the body contact holding him.
- **Reader 2** [M `NDrone2_DealWithObjHit` 0x13fff0:91-93]: on a drone hit of surface class
  ≥ 0xf with `drone+0xc5 == 0x1d`, a bit-0x10 player negates the vector at `drone+0x480`.
- A register-carried-read sweep over every `+ 0x60)` load in ACTION.ELF found no further bit-0x10
  tests; DRIVING.ELF/SFX.IRX have no `+0x60` flag accesses.
- **Port**: no BL+0x60 flags word is modelled — `grounded` is the direct `walkable || band` rule.
  The swim float is modelled behaviourally (the port's swimmer rests at a constant surface y —
  probe-measured). No walk-substate reader of bit 0x10 exists in the game, so nothing wade-side
  is missing.

## The four wade checks (real 07000002 shallows, (-79, 4.68, 28) [L])

Measured with the new `wade_real_live.rs` (real load, real input; the PCSX2 spot settle, then
paced walking within the water line):

1. **Walk speed** — dry 0.0929/tick vs wading 0.0953/tick (within 2.6%, the shelf's slope): the
   game does not slow the wade [M the walk-speed path reads no water state — `Player_SSWalk`
   0x192310 and `Player_ResetWalkAnim` 0x195a88 read neither BLData+0x50 nor +0x60]. The port
   matches; pinned.
2. **Ripples per step** — the game runs `Env_WaterRipple(hit)` on the footstep-frame hit in the
   water pair [M `Player_DealWithObjHit` 0x194458:92-95; the gate `(mat&0x3f)−0xd < 2` at
   `Env_WaterRipple` 0x17c118:9]. The port calls `env_water_ripple` on the same footstep event
   (scene.rs, the [M] block cites) — the test counts the RIPPLE_MESH gas spawns on wade steps:
   ≥ 1. Matches; pinned.
3. **Footprint decals on 0xE** — the disc's 0xE EffectRecord carries `foot_decal_4 = 0,
   foot_decal_8 = 0` [D]; the game's `Effect_Player` reads the same EffectInfo and spawns none
   when the gfx word is 0 [M 0x1cec48:84-89]. The port spawns no decal on the water plane (every
   decal the pacing produced sits on the dry surface the exiting steps landed on — verified
   per-decal against the surface under it). Matches; pinned.
4. **Eye height** — the game's eye = root + BL+0x908 − BL+0x90c [M `Player_GetHeadPos`
   0x195aa0]; BL+0x908 eases toward 0.7 while `surfaced`, 0.0 otherwise [M `Player_PositionCamera`
   0x18f5d8:114-128] — no wade term. The port's eye stays 0.700 through the wade. Matches; pinned.

Also confirmed at the spot: the feet contact byte reads 0xE [L], the footstep sound for 0xE is
0x28 [D `SurfaceSounds`; L], the landing plays 0x5FB (master's `jump_landing_sound_live`, green),
and the deep-water swim entry matches [M `Player_InWater` 0x195e50 — `pos.y − 0.3 < cel+0x90`;
the port's `enter_water` cites it; probe-observed substate 3 at the pond's deep side].

## 07000006 — no reachable wade spot (the trap, confirmed)

The driver's warning checked out and went further. The pond is `Object_Pondwater` (cel #351) at
(27.5, 38.60, −7.00) [D docs/levels/07000006.md]. A plain ray from high above reads 0xE there
(the water plane is the top surface) — that is what produced the "24 shallow cells on 06". But
the GAME's feet ray runs from the standing root, and a 0xC cap sits ~0.01 ABOVE the 0xE plane:
from the root the cap is hit first everywhere. Settle-scan of **625 grid points** (x 9.5-45.5,
z −25..11) with Bond placed and settled: the settled feet bytes are ONLY 0xA and 0xC — never
0xE. (24, −7) itself: scoped feet ray t 0.633 → hit_y 38.61 = the cap; the plain ray from y60
reads 0xE at 38.602 — the two surfaces 0.008 apart.

## Tests

- `crates/nf-view/tests/wade_real_live.rs` — 2 tests, both green: `the_02_shallows_wade_walks_at_
  the_dry_speed`, `wading_steps_spawn_ripples_no_foot_decals_and_keep_the_eye`. Real level, real
  input from the load (the settle teleport follows master's `landing_in_the_02_shallows`
  precedent at the same spot).
- `crates/nf-view/tests/wade_probe.rs` — assertion-free probe: the 02 wade measurements and the
  06 settle-scan tool (the next message's 06 work starts here).
- No game code changed (`crates/*/src/` untouched this task), so the suite risk is nil; the
  neighbors ran green: `jump_landing_sound_live` 3/0, `walkreal_07000002_live` 3/0.

No fix was needed, so there is no red-proof: every check the task lists already matches. The
tests are the pins that go red if a regression breaks any of the four behaviours.

## What's left

1. **The 06 live check** (below): if the game's contact byte at the pond reads 0xE, the port's
   cel scoping drops the ball (the port's buckets are coarser than the game's cels [S,
   documented]) and the fix is a scoping correction followed by a real-level wade test on 06.
   If it reads 0xC/0xA, 06 simply has no wading and the "24 cells" are all the trap.
2. The [S] anim question (live check 3): if the game switches Bond's walk animation while
   wading, the speed parity I measured could hide a cadence difference.

## Live checks for the main agent

1. **06 pond contact byte**: PCSX2, 07000006, stand Bond on the pond at (27.5, ~39.7, −7)
   (on the cap the port reads 0xC from) and also wading at the pond's interior edge; record
   BLData+0x50 (the contact byte) each tick. Expected if the cel scoping diverges: 0xE where
   the port reads 0xC/0xA.
2. **Bit 0x10/0x20 at the 02 shallows**: at (-79, 4.68, 28) record BLData+0x60 — expect 0x18
   (8|0x10) while standing [L given], and whether the 0x20 variant appears (the hit's +0x40 y
   vs BLData+0x760).
3. **The wade walk animation**: at the same 02 spot, record Bond's anim sequence id while
   walking in the shallows vs on the dry shelf — the game may switch to a wade cycle (the
   speed parity would then hide a cadence change the port lacks).

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
this session.

## ntfy failed

All three fallbacks failed (2026-11-06 ~13:25 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "shallow-water-wade: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "... urllib.request ..."` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "shallow-water-wade: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within 30 s (killed at the tool timeout)
