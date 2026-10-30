# level-0c-real-crossings — the report

Task: Chain Reaction Part 1 — the hinge-burn drop and the wire crossings on real input. Replace
each stand-in in `walkreal_0700000c_live` with the real-input leg in the driver's order
(laser-watch fire, jump grabs, strafe rides, drops), fixing any port gap hit [M/D]. Branch
`s/level-0c-real-crossings`, from origin/master 48696227.

## What's done

1. **The hinge-burn leg is REAL and green end to end** (the driver's blocker 2): the walkway
   north to the inner balcony repro spot (7.61, 15.71, −113.39 [L 991]) — the east lane, since
   the walkway carries a raised block x 5..10, z −122..−125 (top 16.49, west face x 10.15
   [D probe, this session]); the Laser (0x4E) equipped through the REAL switch input; all four
   hinge locks #139-142 (out 14..17) burned through the held-fire path (+85..+144 ticks each;
   84 contacts per hinge [M Lock_Update 0x1ae3d8:82-101 + Player_WeaponFiring 0x1a5460:653-658]);
   ch 18 rose (the AND #505 [D]); SS #138/#179 swept and reset (t 0.1 → 3.1 → 0 — the SSs reset
   t/state on completion, so the pin is the END POSE: the y-16.3 ray crossing the open vent
   [M the SS end poses]); Bond walked through the vent and the **trench drop fired plate #274
   MID-FALL at +37 ticks** — ch 0x1b, objective 1 complete on the game's own route ([D #274;
   L hatch-2]). The old vent/pulley stand-in for objective 1 is GONE.
2. **The case-exit walk and the hall/lane route are REAL**: out of the case onto the 11.27 hall,
   the hall east, the upper east lane (11.27, x 13..14 [D probe]) south, the z −152 band west —
   all walked with `step_bond` only (new probe maps this session: the trench-exit floors/rays,
   the 1.28 riser wall at z −114.09, the hall/lane/pit/slab extents, the 6.27 display ground
   running x 12.5..14.5 the whole z −114..−154).
3. **The wire crossings (blocker 4) run as real legs** (from the blockers branch's WIP, carried
   onto this branch and driven further): the jump grab at icon #4 onto wire #284, the strafe
   ride to B; the diagonal ledges to icon #6; the contact grab of #282, the ride east to A; the
   arc to icon #12; the contact grab of #283, the ride to B. **#284 and #282 land inside their
   [D] bands; #283's landing assert is RED — see the honest blocker below.**
4. The photograph, the secret-door chain, the doorway walk, the yard legs, the ladder, the
   ring walks and the band/slot jump all remain the master's real legs and stayed green
   through this session's reroute.

## The honest blockers (what's left, with the evidence)

1. **The wire DROP is unpinned** (new finding, the session's last hour): the rides never
   actually dropped. The `jump` input while riding STANDS Bond UP onto the wire — all three
   WIP "landings" were feet ON the wire line (13.67−0.85 / 14.11−0.93 / 14.44−0.89 = the
   [D] wire ys; #284/#282 passed their y-bands only because the wires sit near the ledge
   heights). The use-key press (4th step_bond arg) does not drop him either; the current tree
   pulses the FIRE action (action 9 — the substates read it directly, like the grapple's
   release [M Player_SSGrapple 0x192860's Input_Action(9,1)]; the jump-instead-of-drop pin is
   in the comments). The last run's #284 read `dropped false` under the use-key variant; the
   fire variant's run is the final tree's. **The wire-drop button needs one live check (below)
   and one more leg iteration.**
2. **The #283 landing assert is red**: the ride ends at B (35.89, 14.44, −209.44) and Bond ends
   GROUNDED at [35.15, 13.55, −209.95] with **no static floor in y 14..10 there [D probe this
   session]** — i.e. he is standing ON the wire (see 1). This is the same landing the
   level-0c-every-objective-blockers-2 task reports against master 531c08fb.
3. **The case's trench exit is unpinned** (new, documented): the case's south glass lintels at
   z −111.89 from y ~13.2 up [D probe rays], the west inner face x 4.78 blocks the chest, and
   the crouched body (root 0.607 [L 05-truck]) still clips the lintel. Glass is body-solid
   [M hatch-4]. No SS keys off ch 0x1b [D keys]. The stand-in (the one this session ADDED)
   moves Bond onto the 11.27 hall with a sister ray pinning the lintel.
4. **The display-ground access is the unported 0xE2 vent/pulley** (pinned harder this session):
   the 11.27 band ends at z −153.14 (side face y ~11.5..13.5 [D probe]) and the pad #506's
   0.57 riser stands behind the same face line — the walker can neither walk off the edge nor
   step up, at x 4.5/5.8/7.6/9.0 alike; the display-eye and plates-reach stand-ins stay
   (master's shapes), now reached over the real hall/lane walk.
5. **The yard descent's east strip wedges on the wall's unprobed east return** (a face at
   x ≈ 55.55 at y ~2 [this run's STUCK]) — the yard stand-in (master's shape) stays.
6. The gate-band crossing stand-in (x 55.5..67) and the ladder495 sister test's west-ground
   placement are unchanged from master (both unpinned mechanisms, documented there).

## What was run

- `cargo test --offline --release -j2 -p nf-view --test walkreal_0700000c_live -- --test-threads=1`:
  the final tree's run is the fire-drop variant (see the log tail); the session's runs walked
  the hinge-burn leg, the hall/lane route, the photograph, the secret-door chain, the doorway,
  the yard, the ladder, the ring and all three wire rides green up to the #283 landing assert.
- `probe0c_lane_live` (this session's scratch probe, NOT committed): the walkway-north lane,
  the trench-exit floors/rays, the south-hall/far-south/under-walkway/east-lane floors, the
  pit-lip rays, the #283 landing rays — every number quoted above.
- The FULL suite: **did not fit the window** (the background fast runner dies at the tool-call
  boundary in this sandbox — three launches, three deaths; the foreground budget went to the
  leg iterations above). Touched crate: nf-view (test-only diff — no nf-game/nf-formats/
  nf-audio source changed, so their suites are unaffected by this diff). Next session: run the
  four-crate suite from this branch (the cache is warm), then land the wire-drop fix.

## Live checks for the main agent

1. The wire drop: fresh boot 0700000c, grab wire #284 at the west edge (icon #4), ride toward
   B, and try each button near the end — which one lets go? (Fire / E-action / jump / nothing
   = auto-drop at the terminus.) Watch Bond's substate and feet height at the drop tick.
2. The case's trench exit: after the four hinges burn and the #274 plate fires, how does the
   game's player LEAVE the display case (the trench x 5..6.5, floor 11.84)? The south glass
   lintels at y ~13.2+ [D probe]; the crouch does not fit [L root 0.607].
3. The display ground: does the game's player ever walk the pit (z −153.14 edge / the pad
   riser), or is the 0xE2 vent/pulley the only route down? gdbwatch ch 0x1b and the pulley
   object on the walk.
4. The doorway-ledge east return: the face at x ≈ 55.55 at y ~2 — what is it (a crate? the
   wall's return?), and does the east-then-south descent walk clean in the game?

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
