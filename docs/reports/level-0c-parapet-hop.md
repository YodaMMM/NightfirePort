# level-0c-parapet-hop — report

Task: the last unmapped hop on 0c's plates route — the rib's west parapet (x −4.84..−4.79, top
8.67, z −112.21..−116.70 [D]) — worked in plates-reach-2's order: (i) scan both parapet ends
for a walk-around at 7.24; (ii) do NOT touch the standing jump (it is EXACT per [L 2026-10-30
PCSX2] — rise 0.7513, apex 22-23 ticks) and look for a crouch/ledge/mantle arm or another link
instead; (iii) otherwise find the real link. Then rewrite leg 4 of
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the
proven chain and no teleport. Branch `s/level-0c-parapet-hop` = origin/master (8ce724ad).

## What's done

**(i) The parapet ends: no 7.24 walk-around exists — proven from the disc.** This window's
`nf-extract coll-scan` dumps over the onfoot ISO (0700000c.bin; session dumps
/tmp/scan_{rib,npar,spar,east,l4,nw,deck,sh,w}.txt, not committed) complete the geometry:

- The parapet is a blade on the rib's west side: top faces y 8.67 (x −4.84) / 8.62 (x −4.79),
  z −112.21..−116.70 [D scan_npar hits 37-42], and its faces rise from y 6.92 (the walkway
  level) up to 11.41 [D scan_sh: v (−4.84 11.41 −113.16) (−4.84 6.92 −113.16) …] — it walls
  every height between the walkway and the ledge.
- The 7.24 band west of the blade (room_sk3's ledge x −5.65..−5.06 + room_storage01's west
  strip x −5.05..−4.54, both top 7.24, z −112.24..−116.47 [D scan_npar hits 5-7, 15-16]) is
  fully covered by the blade's z range: the blade's north tip (−112.21) pokes 0.03 north of
  the band's north edge (−112.24) and its south tip (−116.70) 0.23 past the band's south edge
  (−116.47). **No walk-around at 7.24.** The sim agrees: walking the rib south and pushing
  west wedges at x −4.25 = the blade face −4.84 + capsule radius 0.55 (probe
  `probe_the_rib_west_attempts`, R4).
- The walkway's north part is walled too: the north lane (6.92, x −8.25..−7.34,
  z −110.36..−120.31) has an east wall at x −7.34 (6.92→12.66, z −110.36..−111.86) and a north
  step to a 9.24 ledge (z −110.02..−110.36) walled at z −110.02 (9.24→12.26); the wide part
  (x −7.34..−5.65, z −111.86..−153.52) has a north wall at z −111.86 (6.92→12.25) [D scan_l4/
  deck/nw]. The walkway can only be entered from the east at the ledge (a 0.32 step up from
  6.92 to the 7.24 ledge) — and the blade walls the ledge from the rib.

**(ii) No crouch/ledge/mantle arm exists to change the verdict — the port matches the game
1:1.** Decompile audit (work/decomp/ACTION.ELF): the jump is only reachable from SSWalk
(`Player_HandleJump` 0x1916b0, sole call site `Player_SSWalk` 0x192310:69), impulse
vel = WldGravity × (−0.4) = (0, +3.92, 0) with WldGravity = (0, −9.8, 0) [M Player_Init
0x1910f8:152-153] — rise 0.7513, apex ~23 ticks, exactly the [L 2026-10-30] measurement.
**Crouch-jump does not exist**: from crouch the jump action stands up (`Player_SSCrouch`
0x1924c0:91), HandleJump never runs in substate 4, and the crouch action is only processed in
jump phase 0. **No freeform ledge/mantle/vault exists** (no symtab hits; the only climb arms
are placed ladder objects, class 0x2d, `Ladder_Create` 0x1966c0 — 0c has exactly two, #164
(9.67, 5.53, −187.34) and #495 (−6.38, 0.0, −177.55) [D census], neither near the parapet).
The port reproduces all of it (crates/nf-game/src/player.rs: GRAVITY −9.8, the same impulse,
the 0.95 headroom LOS, the capsule arms, ladder.rs) — **the standing jump needs no fix and got
none.**

**(iii) The real link is found and half-proven: the 6.15 slab under-pass, popped up by the
game's own collision rule.** Under the whole parapet/walkway complex lies room_storage01's
6.15 slab, an L: west arm x −8.25..−5.57 (z −111.62..−116.35) + east arm x −5.57..−4.50
(z −112.58..−116.35) [D scan_rib hits 22-31], and the display ground reaches it: the 6.22
display piece (x −4.41..−2.93, z −112.41..−117.07 [D scan_sh/spar]) steps down 0.07 onto the
east arm, and the old L1 probe lane (z −112.5) missed the slab by 0.08 and stopped at x −5.0 —
the under-pass was never walked. The slab runs WEST **under** the walkway (6.92 top, 0.77
above the slab). The game's own horizontal-face rule — a floor crossed in the hit spine's top
half is skipped (jumped through from below), the same face crossed in the bottom half is solid
and pushes [M Collide_Intersect 0x1ebaa8:321-322; the spine A = pos+0.275, B = pos+0.55−cc,
|d| = 0.7597, Player_Collision 0x194df0 disasm 0x195444/0x195464] — pops a player standing on
the slab up onto the walkway. **Sim-proven**: `probe0c_parapet_live::
probe_the_underpass_pop_mechanics` places Bond on the slab at (−6.5, 7.18, −113.3) and the
PLAIN SETTLE (zero input) lands him at (−6.50, 7.96, −113.3) = feet 6.92 = standing on the
6.92 walkway (the jump-pop variant behaves the same). The headroom LOS gate does not block the
jump variant: the LOS runs pos→pos+0.95 [M Player_HandleJump 0x1916b0:160-165] and the 6.92
plane is below pos (7.18 on the slab).

**The chain up to the slab is re-proven on real input** (probe
`probe_the_strip_jump_past_the_drone_to_the_yard`): the display eye → the west hops → the E1
jump onto the 7.24 platform (LANDED (−4.48, 8.36, −109.22), the proven class) → south-west
onto the wide part's west strip (7.24, x −5.03..−4.54, z −108.67..−111.14 [D scan_npar hits
17-18]) → the strip-end jump south-east LANDED (−3.53, 7.38, −113.87) on the 6.22 piece —
**west of the drone line**, no fall damage. From there the 0.07 step onto the slab, then the
proven pop, then the walkway south → the cross-wall gap lane (x −9) → the west drop onto the
ramp decks → the ramp's south walk-off onto the yard — the same waypoints the pocket_descent
full-route probe walked (K1/D4/F2), replayed in this probe's S8-S16 legs.

## The one blocker left: a dynamic obstacle at x −3.82

Every westward walk at the display level wedges at x −3.82: the z −112.9 lane (2a/2b), the
z −115.2 lane, and the post-strip-jump lane (S6, z −113.0..−113.95) all freeze at
x = −3.8174725, and the blocker is STATIC — four wait-and-retry cycles (4 × 400 idle ticks)
never moved it (bit-identical position every cycle). It is NOT static disc collision: a
coll-scan at (−3.82, 7.0, −114.5) r 1.5 returns no triangle there [D scan_w]. The only dynamic
collision candidate is the sim's oDrone entity (map 11 obj 5): its disc-placement collision
sits at (−3.0..−3.4, 7.16..7.55, −111.5..−111.8) [D scan_sh], but the sim's drone hovers at
(−3.82, 7.38, −113.0) — right on the wedge line. The strip-end jump (S5) lands PAST it
(x −3.53), so the jump direction that crosses its z from the north-east is not blocked; only
the westward ground walks are.

**Leg 4 is therefore NOT rewritten this window** — the entry lane onto the slab is blocked by
the sim's drone entity, and a leg with a blocked lane would go red. The master stand-in stays
byte-identical; `walkreal_0700000c_live` was re-run green (see Verification). The route is now
reduced to ONE question (the drone) plus mechanical wiring of already-proven legs.

## What's left (next window, in order)

1. **Settle the drone** (live check below): does the real game's oDrone hover at
   (−3.82, 7.38, −113.0) permanently, patrol with a gap, or is the port's drone entity
   misplaced/mis-shaped vs the game? If the port's drone is wrong (position/radius), fix it
   with cites [M the drone system; D the placement]. If the game's drone really walls the
   lane, look for the game's own answer (ride its roof — top 7.36, a 0.12 step from the wide
   part — and jump west-south-west onto the slab past it; or a patrol gap).
2. **Rewrite leg 4** with the proven chain once the entry is settled: the display eye →
   west 1-4 → E1 jump onto the 7.24 platform → the west strip → the strip-end jump (S5's
   target (−4.1, −112.8)) → the 6.22 piece → the 0.07 step onto the slab → the under-pass pop
   → the walkway south (S8-S11 waypoints) → the gap lane (x −9) → the west drop (D4) → the
   ramp (F2) → the yard → the section-5 bridge/corridor waypoints → the plates' body touch →
   the E arm. No teleport; the walk must stay green; run every walkreal_* binary if the jump
   code changes (it should not — the jump is exact).
3. **The full suite on the final tree** (see Verification for this window's suite status).

## How it was verified

- Every coordinate above is [D] (`nf-extract coll-scan` over the onfoot ISO, 0700000c.bin) or
  [M] (decompile function + address, quoted in place) or a sim measurement from
  `crates/nf-view/tests/probe0c_parapet_live.rs` (real `step_bond` input after the leg-3
  placement; all four tests #[ignore]d as route records, not pins — no behaviour is pinned
  beyond what was already pinned). The probe's route-record runs: `probe_the_rib_west_attempts`
  (the blade wedge R4), `probe_the_underpass_pop_mechanics` (the pop, P0-P3),
  `probe_the_display_slab_underpass_to_the_yard` (the entry wedge, documented outcome),
  `probe_the_strip_jump_past_the_drone_to_the_yard` (S1-S7 with the wait-retry cycles).
- The touched crates: nf-view only (one new probe file + this report + WORKLOG). No game code
  changed — the jump is exact [L] and no crouch/mantle arm exists [M], so nothing to port.
- **Ran**: `cargo test --offline --release -p nf-view --test probe0c_parapet_live` (builds
  clean; runs quoted above) and the touched mission walk
  `walkreal_0700000c_live` → **test result: ok. 2 passed; 0 failed (15.70s)** —
  `walk_the_whole_mission_from_the_start_to_the_0d_handoff` and
  `ladder495_walk_grab_and_climb_to_the_top` both green on this tree.
- **Suite status (honest)**: the fast runner was started twice and died twice on this
  sandbox's 10 GB disk (disk-full during the nf-view test-binary link phase; the ISO + the
  build cache + the toolchain need ~9.5 GB and the first build ran alongside the sandbox's
  1.2 GB node_modules before it was cleaned). Run 1 also surfaced and I fixed an
  environment gap: nf-audio's `bank_live` could not link (`-lasound` missing) — the ALSA
  runtime lib was extracted into a user prefix (`libasound.so.2.0.0` + `LIBRARY_PATH`), after
  which `bank_live` builds. Given the disk ceiling, the full-suite run on the final tree is
  the next window's item 3; this window's evidence is the direct runs quoted above (the
  branch's only code delta vs master is the ignored probe file — no shipped code changed).

## Live checks for the main agent

1. **The oDrone entity vs the wedge**: in PCSX2 (fresh boot, 0700000c), walk Bond to the
   display ground's west edge around (−3.5, 6.35, −113.0) (or place him there) and record the
   oDrone (map 11 obj 5) entity's runtime position for ~60 s: does it sit at ≈ (−3.82, 7.38,
   −113.0) permanently (then the lane is game-closed too), patrol with a gap, or sit elsewhere
   (then the port's drone placement/radius is a gap — fix with the EE address + values)?
   Expected result decides next window's item 1.
2. **The under-pass pop in the real game**: with the drone settled, walk Bond west on the 6.15
   slab (enter from the 6.22 piece at ≈ (−4.2, 6.3, −113.5), walking west along z −113.5):
   record whether the real game pops him up onto the 6.92 walkway near x −6..−7 (the
   bottom-half spine rule [M 0x1ebaa8:321-322]) or wedges under it. Expected: the pop (the
   port already does it — probe_the_underpass_pop_mechanics), which opens the whole leg-4
   rewrite.

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
