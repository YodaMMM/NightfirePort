# 50-real-health-00agent-14-landing-2

07000014's 00 Agent real-damage walk (`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent`)
crosses the 1.72 landing on master 25d6fc65e and reaches the escape touch; it dies at t29197 at (-30.20, 1.16,
24.28) on the Rook's-wake legs to #30 (skin 0x5000087, 0x14) with 2.9 health. The health gap is the Kiko corridor:
27.9 into the Kiko hold, then #20 (0x5000090, 0x14, hidden at (-51.2, 32.6)) lands 15.0 in 3 at 2.4 u as Bond
passes node 1. The task file of 50-real-health-00agent-14-landing (done/) lists every lever measured on 11-13 -
do not repeat them. Ideas still open: a crouched pass of node 1, a different west-corridor line on #20 before
the Kiko hold, more health out of the west-face fight (#34 10.0). The NF_HOLD_SKIP collision fix (off by default)
also needs its discriminator so it can be turned on: the pit-band jump arc's east gate must survive it.
Real input only, no top-ups; the d1/d2 walks stay byte-identical. Tests: walkreal_07000014_live,
interactables_07000014_live, probe14_pocket_settle. Branch: s/real-health-00agent-14-landing-2.

Measured 11-14: quiet pass to node 2 then a hold there (300/600) dies inside the hold at t26887 at (-47.41,28.27). Not shipped.

Main 11-14 (master 06db4a156): at the Kiko hold Bond holds the 0x14 9+42, owns 0x34/0x37 grenades (1 each in clip, carried slots 22/25 = 4/4); a CROUCHED duel on #20 from the hold spot (300/900) still dies inside it at t26998 at (-51.42,0.65,37.88). Untried: a frag lob onto #20 at (-51.3,32.4) from the hold (5.5 u - check the frag radius first).

Main 11-14: the t26998 death in every "stay near the Kiko hold" variant is #29 (0x5000093, weapon 0x14, hp 14) at (-49.9,32.7): -5.0 x2 then a -17.91 BLAST at frame 26998 - an explosive 0x14 round, not #20. The shipped stream avoids it by leaving for the second hold. A frag lob on #20 from (-50.3,40.0) never got thrown (the 26998 blast first); dodging landed/bounced drone rounds within 8 u in the holds: t27175 (worse). Next: #29 (hp 14) is the Kiko-side threat to answer, not #20.

Main 11-14: the Kiko hold picking skin 0x5000093 (#29) first dies inside the hold at t26493 at (-46.82,42.38) (worse). Not shipped.

Main 11-14: crouching in the holds (one crouch press at each kiko_hold start) - the Kiko hold ends at 22.9 (was 27.9) and the walk dies at t27365 - worse. Not shipped.

Main 11-14: five switch-band stands (radius 15/20, was 3) - death t27276 in the Kiko hold, worse than t29197. Not shipped.

Main 11-14: crouched from the node-618 corridor leg - dies at the door at t25407 (worse). Crouch hurts 14 in every placement tried (holds, corridor).

---
claimed-by: sandbox5 at 2026-11-15T00:06Z

Main 11-14: armour pickups (kind 3) live at the Kiko hold (t26154, Bond 27.9/0 at (-51.3, 37.9)): (-41.3, 1.5, 36.5) at 10.1 u and (-27.3, 0.9, 31.0) at 25 u. A walk_nav to the near one routes SOUTH through the squad (via (-46.6, 28.4)): return fire kills #31/#29/#20/#23/#22 on the way but #61 (0x5000093, 0x14) ends it at t26950 at (-45.0, 28.0) - worse than t29197. Not shipped. The stock room at x -32.2 (z 31-36) holds weapon 0x1a/0x34/0x3a and ammo 0x15/0x1a/0x1b.

Main 11-14: the escape-room armour (-27.3, 0.9, 31.0) right after the escape cut-scene: Bond arrives there with ~3 health, #63 (0x5000090, 0x14) at 4.7 u ends it at the room door (-30.6, 29.9) at t28980 - worse. The health is spent in the holds before the touch.

Main 11-14: the biggest single bleed is the south loop corner (nodes 42-44, t11354-11395): Bond walks into #14/#15 (0x14) at 1.0 u, 35.0 in 40 ticks. Measured stops: at node 41 (-21.4, -21.1) four 400-tick rounds never land on #14 (11 u, nothing exposed), the gun runs dry and the walk dies in the Kiko hold (t40789, a stall-inflated tick, NOT further along); at node 42 (-22.48, -19.97) both die (#14 at 9.8 u, #15 at 4.5 u) at 50.4 health (master 37.9) - but the Kiko corridor is reached at the same 27.9 and a -22.91 blast from the already-dead #28s explosive round ends it at t26516. Not shipped; the corner fight saves 12.5 that the later stretch loses again.

Main 11-14: the Kiko corridor fight as a 600-tick kiko_hold (nearest visible, grenade step-away) instead of the 4 x 300 stands: t26765; with the node-42 corner fight too: t26519 - both worse than t29197. Not shipped.

---
## Report (done, 2026-11-15T02:00Z)

Closed by the driver: the report is on `s/real-health-00agent-14-landing-2` (docs/reports/real-health-00agent-14-landing-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-15T02:03Z)


# real-health-00agent-14-landing-2

Branch `s/real-health-00agent-14-landing-2`, cut from origin/master 272ec32c8. The task:
07000014's 00 Agent real-damage walk
(`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent`)
crosses the 1.72 landing and reaches the escape touch; it dies at t29197 at
(-30.20, 1.16, 24.28) on the Rook's-wake legs to #30 (skin 0x5000087, 0x14) with 2.9
health, and the health gap is the Kiko corridor (27.9 into the Kiko hold, then #20 lands
15.0 in 3 at 2.4 u as Bond passes node 1). The open levers named: a different
west-corridor line on #20, more health out of the west-face fight (#34 10.0), and the
NF_HOLD_SKIP discriminator.

## What's done

**1. The baseline re-verified byte-identical on this master (272ec32c8).** The ignored
00 Agent walk dies at t29197 at (-30.20, 1.16, 24.28) to #30 with 2.9 health, exactly the
ledger's shape [L the run, 37 s]. The walk crosses the landing, the drop west and the whole
descent cleanly — the task's premise (the t29197 death on the Rook's-wake legs) is the
current wall, not a new one.

**2. The NF_DMG_TRACE baseline decoded the whole Kiko-corridor ledger [L this tree].**
- Kiko hold ends at 27.9.
- #20 (0x5000090, hidden at (-51.2, 32.6)) lands 15.0 as 3 × 5.0 at t26993-27001, Bond at
  (-48.8, 32.0), d 2.4 — the node-1 point-blank pass; Bond's return fire kills him at t27024.
- #23 2.5 (t27032), #22 2.5 (t27238) → 7.9 into the second hold.
- #31 5.0 in 2 (t27576/27580, (-34.1, 26.8), d 4.6) on the final run → 2.9 at the touch.
- The west-face 10.0 the task names is #32 (skin 0x5000093, hp 20, weapon 0x14): frames
  25307/25311, shot from (-56.5, 40.1) at d 7.8, DURING the cascade/door wait — the leg's
  own return fire kills #34/#32/#33 only at t25385-25456, after the hits.
- The 0x5000093 skin carries the explosive 0x14 (the whole family: #29, #32, #33 fire it).

**3. Six levers measured, each on its own real run — all dead, none shipped.**
1. *Second-hold pocket stands* (after the 300 hold, 3 × 300 on the nearest visible guard
   inside 15 u — the corridor stands' own law): death t28200 (worse) at (-35.28, 27.61),
   2.3 u short of the touch; the stands re-roll the pocket and #62 (0x5000087) plus #29
   (0x5000093) converge on the final leg.
2. *West-face answer after the west-face walk_to* (2 × 300): the walk_to's return fire
   killed #34/#32/#33 — death t29197 byte-identical, zero health gain (the 10.0 landed at
   t25307 before any answer could stand).
3. *Cascade-end answer* (right after ch 0x85, t24886): NO-OP — nobody visible from the
   band at the cascade break (the pick breaks in 0 ticks); run byte-identical, t29197.
4. *Door-wait answer, gun not switched* (inline during the 180-tick wait): the stands shot
   the Q-Decryptor 0x4e (clip 500 — a tool whose rounds never draw blood) — death t25344,
   much worse. Measured lesson: `switch_to_armed` accepts the 0x4e beam charge as ammo; the
   fight law's `switch_to_gun` (2..=0x47) is the correct gate.
5. *Door-wait answer with `switch_to_gun`* (real gun, clip 13 spare 63): death t25329 —
   standing in the open at the band draws the burst; the window is a killing field either
   way.
6. *The north-lane crossing past #20's pocket* (the task's own open lever —
   `(-49.6, 36.2)` → `(-46.4, 34.8)` → the second hold, keeping #20 outside the 3.0 u
   auto-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115]): **#20's 15.0 was
   AVOIDED ENTIRELY (0 #20 hits [L the run])** — but #29 (0x5000093, at (-49.1, 30.2))
   fired ONE explosive round that landed **-27.91 at d 6.1** (t26998) on the whole 27.9.
   The z 36-37 band is #29's kill line. Plus the hold-side variant (093-first picks,
   300-cap stands): death t26493 inside the hold — reproduces the 11-14 measured result.

**4. The shipped stream is kept and re-verified.** All six levers reverted (the final tree
carries no code change — the walk file differs from master only in the refreshed ignore
ledger). The d1/d2 walks are untouched: every lever sat behind
`difficulty >= 3 && !tops_up()` gates, and the final tree's d1 walk and d2 sister are
byte-identical to master's (they share no gated code with the ignored 00 Agent body).

**5. A seventh lever was measured after the first ledger push: the middle-band crossing
(z 34.4), named in the first report as the next lever — dead too.**
- "middle lane south past #20's pocket" → "middle lane east to the band head"
  (z 34.4/33.8): death **t27096** at (-48.91, 35.44). #20 TRACKS east on the re-roll (he
  moved to (-50.5, 32.2)) and hit 3/3 rounds at 3.6 u — the Agent accuracy at 3.6 u is
  effectively point-blank; #29's explosive connected IN FLIGHT (t26998) even after Bond's
  return fire killed him at t26891; #23 added 2.5 + 2.42.
- Conclusion, now measured three ways: the corridor cannot be crossed by line choice
  alone — the north lane (z 36.2) dies to #29's kill line, the middle band (z 34.4) dies
  to #20's tracked fire, the shipped node-1 pass dies to the 2.4 u point-blank. Reverted;
  the whole binary re-verified 9/0/1 after the revert.

**6. The next lever is named with its numbers (pushed into the ignore message).**
- The *frag lob* the 11-14 note left open, now with its radius math and a precondition:
  the 0x34 frag is **radius 8 / damage 100** [M 0x12cd98/0x12c5a0 records], #20 (hp 10)
  sits 5.3-5.5 u from the hold spot, the fuse is 210 ticks, and the thrower must EXIT the
  8 u band before it pops — the shipped route east passes 2.4 u from the pocket inside the
  fuse window, so the lob needs a west/doorway retreat or it kills Bond too. PRECONDITION:
  the grep only confirmed the DRONE throw path (`DroneWeap_ThrowGrenade` 0x179558 via
  `pending_grenade`, drone.rs); the port's PLAYER grenade throw must be verified to exist
  before this lever is measurable.
- The *west-face 10.0* (#32's frames 25307/25311 land inside the door-wait window where
  every stand placement died — t25329/t25344): needs a COVER answer, not a stand.
- The NF_HOLD_SKIP discriminator (the pit-band jump arc's east gate must survive it) is
  untouched this round — the walk's wall is the health gap, not the landing.

## How it was verified

- `walkreal_07000014_live` (whole binary, --test-threads=1): **9 passed, 0 failed, 1
  ignored** [L 175 s, the ignore is the 00 Agent walk's own refreshed ledger].
- `interactables_07000014_live`: **10 passed, 0 failed** [L 13 s].
- `probe14_pocket_settle`: **1 passed (LEG A + LEG B)** [L 2.7 s].
- Seven lever runs, each judged on its own real run: t28200, t29197 (=baseline), t29197
  (no-op), t25344, t25329, t26998 + t26493, t27096 — every one worse than or equal to the
  shipped t29197, so the shipped stream stays.
- The full `cargo test` suite was not run: the final tree carries NO `crates/*/src`
  change (only this walk file's ignore string + docs), so there is no source delta for
  the suite to judge beyond the three binaries above; the 37 s-per-run walkreal binary
  covers the d1/d2 twins byte-identically.

## What's left

- The walk's real wall: the Kiko corridor's health gap — the middle-lane crossing and the
  frag lob (with the self-blast answer) are the named next levers, both now measured to
  the number in the ignore message.
- The NF_HOLD_SKIP discriminator (velocity/progress regime so the pocket grounds by
  default without costing the pit-band jump arc) — carried from the 11-14 report, still
  open.
- The 0a recording re-record (NF_REC_0A_WRITE=1) on master — pre-existing drift, not this
  branch's.

## Live checks for the main agent (PCSX2, fresh boot)

1. #29's explosive round: stand at (-49.7, 1.05, 36.3) in the Kiko corridor with the
   0x5000093 guard alive at (-49.1, 1.0, 30.2) — confirm the real game's single-round
   damage vs the port's -27.91 (the blast is the one-hit class; if the real game's falloff
   is gentler, the north-lane lever reopens).
2. #20's hidden spot (-51.2, 1.0, 32.6): confirm he does not fire on a z 34.0-34.5
   crossing (the middle-lane lever's premise — the real game's sightline vs the port's).

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token,
cost or context-window readout to the worker. What was visible: the task message's clock
(sandbox expiry ~03:11 UTC; this report written ~02:00 UTC), `git` hashes, test outputs
and process exit codes. No input/output token counters, no credit balance, no dollar cost.
