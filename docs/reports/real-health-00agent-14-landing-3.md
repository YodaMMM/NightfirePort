# real-health-00agent-14-landing-3

Branch `s/real-health-00agent-14-landing-3`, cut from origin/master f3c20899a and merged with
origin/master 4f3802517 (a 07000008 walk change, no 07000014 overlap) before the final runs.
The task: 07000014's 00 Agent real-damage walk
(`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent`) dies at
t29197 at (-30.20, 1.16, 24.28) on the Rook's-wake legs to #30 (skin 0x5000087, 0x14) with 2.9
health — the touch reached at 2.9 — and the health gap is the Kiko corridor (27.9 into the Kiko
hold, #20's 15.0 at the node-1 pass). This window's lever: measure the middle-lane crossing
(z 34.0-34.5) the landing-2 report proposed — #29's sightline from (-49.1, 30.2) to that lane,
while keeping clear of #20's pocket — then carry the walk on.

Note: `origin/s/real-health-00agent-14-landing-2` no longer exists (the driver closed it after
reading its report); the landing-2 report survives in the queue's
`done/50-real-health-00agent-14-landing-2.md` and that is what this round built on.

## What's done

**1. The baseline re-verified byte-identical on f3c20899a [L the run, 37.6 s].** The ignored
00 Agent walk dies at tick 29197, last hit -2.91 from #30 (0x5000087, 0x14) at (-31.9, 1.0,
27.3) d 3.5, on 2.9 health — 43 hits for 100.0, 17 return-fire fights. Same shape, same tick,
same killer as the ledger.

**2. The middle-lane crossing (z 34.0-34.5) is CLOSED BY GEOMETRY — measured, pinned in
`probe14_kiko_sight` (3 green tests on a deterministic d3 load).**
- The corridor squad sits HIDDEN at load [D placements; L the probe]: #20 (0x5000090) at
  (-51.3, 32.4) — the pocket, #21 (0x5000087) at (-47.9, 33.5) — the east machine cover,
  #19 (0x5000087) at (-45.9, 27.3); #29 (0x5000093, the explosive-0x14 shooter) is scripted in
  later, not a load placement.
- The hold→second-hold nav route is exactly (-48.64, 0.40, 32.13) → (-48.63, 0.40, 30.49)
  [L the probe]: the shipped "node 1" pass is d ~2.7 from #20's pocket — the 3.0 u auto-hit
  band [M DroneWeap_DoBulletAccuracy 0x179740:113-115] — the measured 15.0.
- **#29's sightline from his measured firing spot (-49.1, 30.2) is CLEAR across x -50..-48 at
  every z 34.0-34.5** (d 3.8-4.4 — CLOSER than the north lane's measured 6.1 kill at z 36.25);
  blocked only west of x -51 (t 0.44-0.9) and east of x -47 (t 0.65-0.8). The middle lane is
  NOT protected from the explosive round by geometry.
- **#20's pocket sightline from (-51.2, 32.6)**: blocked to x -52..-50 at z 34.0-34.5 (his hide
  cover, t 0.21-0.28 — the half of the middle-lane premise that DID hold), CLEAR to x -49..-48
  (d 2.6-3.7), blocked east of x -47; the north lane (z 36.2) is blocked from the pocket too
  (t 0.11) — the north run's 0 #20 hits confirmed geometrically.
- The only z 34.0-34.5 band clear of BOTH sightlines is x ≤ -51.5 — inside #20's pocket itself,
  inside the 3.0 u auto-hit band — or the east cover at x ≤ -47, which sits d 1.3-1.5 from
  #21's hidden spot. **No walkable middle-lane line keeps clear of both shooters**, and the
  guards TRACK (the landing-2 z 34.4 run's t27096 death: #20 re-positioned east and hit 3/3 at
  3.6 u). The lever is closed without another 37 s run; the walk stays on the shipped stream.

**3. The frag-lob precondition is NEGATIVE — measured this window.** The port does NOT
implement the player's grenade throw: player.rs/gun.rs carry the 0x34/0x37 as ammo bookkeeping
only (clip[0x45], the grenade-belt pseudo-weapon [M 0x1a16c0:14-15; gun.rs's no-ammo checks]);
the only throw path in the port is the drones' (DroneAnim_EventFunc datum 1 → pending_grenade →
DroneWeap_ThrowGrenade 0x179558 [M drone.rs]). The decompile has no separate player-throw
function — the throw rides the shared DroneWeap family. So the lob the 11-14 note left open
("the 0x34 frag is radius 8 / damage 100 [M 0x12cd98/0x12c5a0], #20 hp 10 and #29 hp 14 sit
5.3-5.6 u from the hold spot, the fuse 210 ticks, the thrower must exit the 8 u band") is
blocked until the player's throw exists.

**4. The next lever, with its numbers, is in the ignore message.** Implement the player's
grenade throw the game's own way — the anim-state 0x49 throw release feeding
DroneWeap_ThrowGrenade 0x179558's arc/fuse, the law the drone path already runs — then the lob
from the Kiko hold (-51.3, 37.9) onto the pocket kills #20 AND #29 inside radius 8 / damage
100 (both 5.3-5.6 u from the hold spot), with the thrower retreating west through the opened
door #151 out of the 8 u band before the 210-tick fuse pops (the hold spot is d ~5.5, inside
the band). That removes the 15.0 AND the blast threat in one action; the remaining bleed on
the escape legs (#23 2.5, #22 2.5, #31 5.0) leaves the touch above 20 — past #30's 2.9 kill
with headroom.

**5. The shipped stream is kept; the d1/d2 walks untouched.** The final tree carries no
`crates/*/src` change and no common/ helper change — only the walk file's refreshed ignore
ledger and the new probe file. The d1 walk and the d2 sister share no gated code with the
ignored 00 Agent body.

## How it was verified (final tree = e62dc56eb merged with 4f3802517 → fcd4ee7be)

- `walkreal_07000014_live` (whole binary, --test-threads=1): **9 passed, 0 failed, 1 ignored**
  [L 170.6 s on the merged tree; the ignore is the 00 Agent walk's own refreshed ledger].
- `interactables_07000014_live`: **10 passed, 0 failed** [L 14.5 s on the merged tree].
- `probe14_kiko_sight` (new): **3 passed, 0 failed** [L 4.2 s on the merged tree].
- The baseline lever run (37.6 s) matched the ledger byte-shape before the probe work.
- The full `cargo test` suite was not run: the tree carries no `crates/*/src` or shared-helper
  delta for the suite to judge beyond the three binaries above, and this sandbox kills
  background processes between tool calls (verified twice), so the long suite cannot run
  detached here. The 07000008 merge re-ran all three 07000014 binaries above on the merged
  tree; the 07000008 walk binary itself is master's own tested change, not this branch's.

## What's left

- The walk's real wall is now a FEATURE gap, not a route choice: the player's grenade throw
  (the ignore message's next lever) — without it, every corridor crossing dies to #29's
  explosive or #20's tracked point-blank, measured three ways plus this round's sightline
  geometry.
- The NF_HOLD_SKIP discriminator (the pit-band jump arc's east gate must survive it) — carried
  from 11-14, untouched; the walk's wall is the health gap, not the landing.

## Live checks for the main agent (PCSX2, fresh boot)

1. #29's explosive round falloff: stand at (-49.7, 1.05, 36.3) in the Kiko corridor with the
   0x5000093 guard alive at (-49.1, 1.0, 30.2) — confirm the real game's single-round damage
   vs the port's measured -27.91 at d 6.1 (if the real game's blast is gentler at that
   distance, the north-lane lever reopens).
2. #20's hidden spot (-51.2, 1.0, 32.6): confirm he does not fire on a z 34.0-34.5 crossing
   before he re-positions (the middle-lane lever's premise, measured blocked in the port —
   the real game's sightline vs the port's; the port's answer is pinned in probe14_kiko_sight).
3. Player grenade throw: with the 0x34 frag equipped (clip 1, carried slots 22/25), press fire
   at the Kiko hold — confirm the real game plays the throw (anim state 0x49, the release event,
   DroneWeap_ThrowGrenade 0x179558's arc) and the frag lands ~5.5 u out with a 210-tick fuse.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost or
context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~06:11 UTC; this report written ~05:05 UTC), `git` hashes, test outputs and process exit
codes. No input/output token counters, no credit balance, no dollar cost.
