## Report (real-health-00agent-03-captors-6, 2026-11-15)

Task: continue the 07000003 00 Agent full-route real-damage walk (still ignored). The walk is
NOT green; the ignore message on the branch (commit 5f5dbbeb6) carries this round's full ledger
and the next levers. Two of the -5 ledger's blockers FELL (the trio wake race and the arm); a new
one (the door-8 mouth's own wake popping #14) is the hand-off.

### What was done

Branch base: origin/master with origin/s/real-health-00agent-03-captors-5 merged in (its lever
commit bbde5ed94 carries the -5 ledger). All runs are real-input d3 walks of
`walk_the_full_route_and_complete_every_objective_real_damage_00agent`, each lever judged on its
own run, no top-ups, damage/health/ammo untouched:

- **Run 29 (baseline) + the diagnostic run 30** — the run-28 shape exactly: death 73318, #5
  (shotgun) at d 1.2, 39.4 in 16 hits, the trio at FULL hp. The run-30 prints showed the wake
  sweep's 600-tick budget found NOBODY because the trio's scripted wake walk ends at the
  east-lawn door (22.3,5.3,10.9) and that door stands CLOSED — the trio bunches BEHIND the leaf
  and the single eye ray never reaches their boxes; they crossed only when the servant nav's
  wedge E-opened the door mid-walk [L d3run29/30]. The wake sweep never fired a shot.
- **Run 31 — the -5 ledger item 2's colder mouth, tried and REFUTED.** With the pre-crossing
  stands dropped the crossing itself is lethal: death 10009, 100.0 across 9 hits, 5 fights — the
  fights are OFF across the crossing legs by the run-9 doctrine, so the lane cluster's glass fire
  ran unanswered. The pre-crossing stands STAY (48.0 is the measured price of the #11+#17 kills).
- **Run 32 — the door funnel v1** (own-door E-open + the (22.6,8.0) standoff): the trio does NOT
  pour through — they HALT north of the open door and duel through the opening; Bond's eye ray
  from the standoff exits the doorway band laterally (the ray to (26.5,16.7) crosses the wall
  plane at x 23.47) and every plain-fight tap is unseen-gated off (the fight on #4 ran 436 ticks
  killed-false). Death 73567.
- **Run 33 — the duel line (22.3,9.6)**: on the doorway sight line all three ARE exposed — the
  sweep engaged #5, #4, #3 back-to-back, killed #5 and #4, DRY-bailed on the 24-hp captain at
  +18, and the loot walked DRY Bond to the drops under #3's machine gun (41.2 in 6 hits at
  d 4.3). Death 73847.
- **Run 35 — the wheel/pickup diagnostic**: the trio's drops (kind 0 at (24.84,14.35)/(26.64,16.16))
  sat UNCLAIMED inside the loot band — the loot's nav stopped 1.29 u short and the walk-over take
  never fired; the wheel carried no 0x1c/0x12 entries [L d3run35].
- **Run 38 — the loot-decline diagnostic**: the loot helper DECLINED on EVERY lane/moat leg
  because the lane kills' guns sit in the moat ditch below the lane's east edge — (14.50,5.35,
  -29.03) dxz 4.2, (14.19,5.34,-28.02) dxz 3.6, (9.66,5.52,-33.13) dxz 4.5, the ledge gun
  (14.01,8.34,-28.12) dy +1.9 — all inside the 8-u/±1.5-y band, all declined by the sight filter
  (the ditch lip blocks the eye ray from the lane floor). #7's drop confirmed at (11.41,9.09,
  -15.55), y 9.09 — the upper floor, outside the band from the lawn [S→L the -5 ledger's
  unverified item].
- **Runs 40/42/43 — the ditch scoop** (`scoop_drops_03`: no sight filter, nav + a
  proximity-gated direct finisher, lenient, failed-target skip — the west ditch's void-edge drop
  (2.55,5.34,-29.39) is unreachable and must never walk the direct leg into walk_to's
  unconditional 6000-tick panic [walker.rs:1381]): the nav descends into the ditch and TAKES the
  guns (Bond even climbed to the ledge gun). The wheel at the wake went from 11 rounds on one gun
  to 0x12:7 + 0x2:4 + carried pools [3:4 9:6 23:2].
- **Run 43 — THE TRIO WAKE IS WON**: with the arm in hand the serial sweep killed ALL THREE —
  the post-wake print shows #3/#4/#5 all corpse true — at ZERO damage taken during the wake (the
  duel line keeps the halters out of the under-3.0 point-blank band [M DroneWeap_DoBulletAccuracy
  0x179740:113-115]). The run then died LATER: 67488 at (35.96,4.27) at the door-8 mouth.
- **Run 39 — the interleave variant, tried and rejected**: a post-stand gap (kill #5 only, then
  scoop) let #4/#3 pour 41.2 through the doorway while Bond levelled the aim — keep the stands
  SERIAL (keep_aim) as runs 37/38 measured.
- **Runs 45-48 — the door-8 mouth approach, tried and measured**: #14 (skin 0x5000039, weapon
  0xe, 12 hp, load DISABLED (30.7,6.2,4.1) [L the run-35 census], enabled false through the whole
  trio fight) pops when the mouth route crosses the door-8 area's own wake (the (39.0,12.9)
  door's open channel and/or the (38.9,11.1) 0x23 plate), closes to d 2.5 and bursts 41.2 in 3
  hits (one 24.0 a round) — death 67488 in runs 43/49 with the plain return fire killed-false on
  him. The south legs z 5.7-6.8 wedge on the garden wall corner at (26.0,7.7) (walk_to's
  unconditional 6000-tick cap panics — run 47); as nav hops the A* rides the north lawn straight
  into the plate (run 48: death 67779); the leg-0 aimed sweep found #14 unexposed behind the
  building's west face (run 46). Reverted to the original A* mouth walk + the d3 wake sweep after
  the settle loop (run 49 = the run-43 shape, the ledger's open item).

All new/changed behaviour is gated to `difficulty >= 3` inside
`crates/nf-view/tests/walkreal_07000003_live.rs` — no shared helper
(`crates/nf-view/tests/common/`) was touched, no `crates/*/src/` change at all. The loot
helper's temporary decline-diagnostic was removed before the push; the walk's own [03-ammo]
prints (d3-gated, stderr, the same class as the master's [03] servant prints) stayed as the
continuation's instrumentation. The refreshed ledger is in the ignore message (commit 5f5dbbeb6).

### Verification (what ran, with pass counts)

- walkreal_07000003_live WHOLE on the final tree (merged with origin/master through the
  captors-5 merge): **6 passed / 0 failed / 2 ignored** (222.58 s) — the byte-identity proof for
  every other stream in the binary (the d1/d2 walks and the replays stayed green; the ignored =
  the d3 walk + the door-8 probe).
- interactables_07000003_live WHOLE: **9 passed / 0 failed** (30.35 s).
- The FULL suite (tools/test/suite.py) did NOT run: the sandbox kills background jobs between
  tool calls, the suite's build phase alone exceeds the minutes left after the runs above, and
  the four-crate build ran in foreground chunks for the two named binaries instead. No test
  FAILED in any run. Diff scope: only walkreal_07000003_live.rs (+ docs), so the two named
  binaries cover every test using the touched code.

### Live checks for the main agent

- The door-8 area's wake source: does the (39.0,12.9) door's OPEN channel (not a plate) enable
  #14 (skin 0x5000039, load disabled (30.7,6.2,4.1))? Poke: stand Bond at (39.4,6.4,12.5), E the
  door, watch #14's enable flip in the same tick. The (38.9,11.1) 0x23 plate is the alternative.
- The garden wall corner at (26.0,7.7): the direct east-lawn legs wedge there from the saddle's
  east side. Is there a real-input walkable line from (23.5,8.4) east to (28.5,6.8) that a player
  takes (the si==0 flow walks the same legs green — but from the west, after the captor-room
  rally)? A render at --bond 26.0,6.4,7.7 would settle what the wall's footprint really is.
- The moat ditch at (14.2-14.5, 5.34, -28..-29): the ditch guns are reachable by the port's nav
  (Bond descends and takes them). Does the REAL game's Bond stand on that ditch floor, or is the
  port's nav cel set too generous there?

### What's left

- The walk (the refreshed ledger in the ignore message is the hand-off): (1) the door-8 mouth
  approach — reach (35.6,3.4) (or fight the cluster through door 8 from (36.2,5.7)) WITHOUT
  crossing the door-8 wake zone; the si==0 legs walk the same lawn green from the WEST — try the
  d3 block-order swap (the cluster mouth fight BEFORE the trio wake) or the west-side approach
  from the si-1 anchor. (2) the #14-awake answer (the exposed sight line or the heavier gun in
  hand before the mouth walk). (3) the older levers: the 87000-tick (35.6,8.7) door stall, the
  exit-lawn #16 post-kill volley, the upper-floor climb for #7's drop.

### Usage

not visible to me
