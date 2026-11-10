# los-mask-repins-0c — the 0c walks under the game's guard sight, round 2

Branch s/los-mask-repins-0c from origin/master 114737d6d. Task: re-route the three 0700000c
walks the game-correct sight moved red (the handoff, its real-damage sister, the ground route),
check the round-3 note (the landing shots on the strip guard), remove each ignore when green.

## Done this round

1. **The 0c-diag evidence dump** (walkreal0c.rs, read-only, runs at the #284 landing on every
   shared-prologue walk): the strip guard #35 (0x5000087, 10.0 hp, team 1) IS at
   (-7.07, 12.24, -218.60) in AimStandFire 0x68 when the walk arrives [L the run] — the
   ledger's position holds on this tree; and the deep map of the south-east surface.
2. **The deep map [L the 0c-diag dump]**: south-east of the landing the walkable surface is
   thin wall tops over a broken 11.24-11.38 lattice — a ~3-u diagonal band
   ((-9,-217)..(-8.5,-218.5)..(-7,-219.8)..(-5,-220.5)..(-2,-223)..(0,-224)) with void between
   the patches; NO continuous surface in 11.8..13.4 south of z -219.5. The "y 12.4 lip" the
   earlier walks rode is a narrow wall top (the walker stands at 12.37 where the grid probes
   read 11.24 half a unit away).
3. **Repairs tried and reverted** (each measured, none green):
   - The -4 branch's lip route (west lane (-8.4,12.4,-218.8) then the brush-past slide to
     (-7.3,12.4,-219.6) r 1.2): the west lane HELD (arrived (-8.36,12.37,-217.82) [L]), but
     the slide carried Bond off the lip's south-east edge to the YARD (740 ticks, ended
     (-6.30,1.96,-218.95) [L]) — on the -4 tree the same slide stopped inside the arrive
     radius; the guard's firing footwork differs under the sight era.
   - The floor route (step off the lip west of the guard, pass south of him on the 11.24
     lattice): the step-off legs held the lip (the walker will not descend a thin wall side —
     the STUCK dump: segment_to_target blocked at 0.45 of 1.94, standing at 12.38) and the
     next leg fell to the yard y 1.96 at (-8.78,-218.26) [L].
4. **The round-3 note (the landing shots)**: not settled. The run-time guard position is only
   reachable in-sim (he walks to the strip from his (-19.70,12.24,-217.33) spawn); the spawn-
   list probe (crates/nf-view/examples/probe0cshot.rs, scratch, not committed) cast the port's
   rays at the SPAWN spots, which is not the walked geometry. The -4 session's measurement
   stands: the walk's own fights read aim-Some while the rounds die on the ledge lip [L that
   session] — [S] the miss is the lip, pending an in-sim ray cast at the run-time stand.
5. **The ledgers updated** (all three walks): the ignore messages now carry this round's
   mechanism + the next lever (map the lip-top path west of the guard's capsule — the west
   lane leg held here — then a guarded slide east; or settle the landing-shot question in-sim
   and kill him from the landing if the game allows it).

## What ran (release, --test-threads=1, final tree 423f19a9f)

- walkreal_0700000c_live: **1 passed, 0 failed, 3 ignored** (the three ledgers above).
- walkreal_0700000c_eastroute_live: 1 ignored; walkreal_0700000c_barrelrow_live: **1 passed** (8.2 s).
- interactables_0700000c_live: **7 passed, 0 failed** (14.0 s).
- The other walkreal0c.rs consumers: guard1_rounds_0700000c_live **2/0** (21.6 s),
  comms_chatter_0c_live **1/0** (13.6 s), upward_shot_0700000c_live **2/0** (16.2 s).
- The full four-crate suite did NOT run: this sandbox kills every background job between tool
  calls (plain nohup AND setsid died within seconds — the same wall the pretrans-arms-5 round
  hit), and the foreground build does not fit the 9.9 GB disk (the linker bus-errors at 100%).
  The chunks above are the coverage stand-in.

## What's left (the next round's work)

- The strip crossing itself: the lip-top path west of the guard's capsule, mapped from the
  0c-diag dump (the west lane leg (-8.4,12.4,-218.8) r 1.0 is proven on this tree; the missing
  piece is the east continuation past his capsule — a slower slide with a hold-and-wait for
  his AimStandFire cycle, or an in-sim shot check first).
- Then the #282 ride release (the -4 tree landed the ride on an isolated 13.16 tier at
  (4.98,-223.29) — the release-point lever: ride to the natural stop past A and jump there,
  the #283 ride's own technique) and the SW arc legs.

## Live checks for the main agent

1. [S] In the real game at 0700000c, stand on the #284 landing (-8.2, 12.8, -215.7) and fire
   the hand gun at the strip guard when he holds the strip: does a round land? (The port's
   fights read aim-Some/rounds-die-on-the-lip; the game's answer settles whether the re-route
   must go around him or can shoot him.)
2. [S] Watch the strip guard (0x500087) from the landing: does he hold (-7.07,12.24,-218.60)
   in AimStandFire, and does his footwork shift while firing (the port's slide outcome changed
   between trees with his firing)?

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
