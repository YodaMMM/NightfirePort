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
4. **The round-3 note (the landing shots) — ANSWERED: the miss was the ammo, not the world.**
   The in-sim ray casts (the 0c-diag block, run at the #284 landing against the ACTUAL strip
   guard): the eye rays to his feet/torso/head AND the chest-height rays from every forward
   offset 0..1.6 u are ALL CLEAR [L the 0c-diag run; the port's collision is the disc's, so
   the game's bullet reaches the same [M]]. The shots still never landed — and the shot
   telemetry found why: the gun arrived at the landing with **clip 1, spare 0** (weapon 0x54);
   the single round fired OFF-TARGET (the aim was still converging), and every on-target
   trigger pull after that was a dry click [L the telemetry run]. The -4-era "600 ticks, hp
   10.0 never moved" was the same dry gun (the walk's earlier fights had burned the ammo).
   A bounded landing-shot attempt stands in the walk (2 attempts, the repro) with the
   telemetry eprintln.
5. **The ledgers updated** (all three walks): the ignore messages carry the crossing
   mechanism AND the round-3-note answer, with the next lever: **bring ammo** (a resupply pass
   before the #284 ride — the walk's own pickup points), kill the 10-hp guard from the landing
   with the on-target shot (the world allows it), and the straight -3 diagonals walk clean to
   the grab; then the #282 ride release lever.

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

- The shot route first, with the fetch's own measurement in hand: the round-2 fetch walk
  reached the west area's ONLY pickup (kind 0x0003, weapon 0x0006, 50 rounds at
  (-21.53,11.33,-218.54) [D the pickup dump; the whole west filter matched one]) and the
  wheel still read weapon 0x54 clip 1 spare 0 after the switch cycle [L the run] — read
  Pickup_Handler 0x18e650's equip path for why (owned-already? kind semantics?), or keep the
  START ammo (the loadout [1,2,30,74,78,80,84]) by fighting the ring leaner, so the landing
  shot has rounds; then the on-target shot kills the 10-hp guard and the straight -3
  diagonals walk clean to the grab (the pre-solid era's proven path).
- Fallback if the ammo detour stalls: the lip-top path west of the guard's capsule (the west
  lane leg (-8.4,12.4,-218.8) r 1.0 is proven on this tree; the missing piece is the east
  continuation past his capsule — hold-and-wait for his AimStandFire cycle).
- Then the #282 ride release (the -4 tree landed the ride on an isolated 13.16 tier at
  (4.98,-223.29) — the release-point lever: ride to the natural stop past A and jump there,
  the #283 ride's own technique) and the SW arc legs.

## Live checks for the main agent

1. [S] In the real game at 0700000c, stand on the #284 landing (-8.2, 12.8, -215.7) WITH AMMO
   and fire the hand gun at the strip guard (-7.07,12.24,-218.60): does a round land? (The
   port's world rays are clear at every height; the port's own shots were fired DRY. The
   game's answer confirms the shot route before the next round builds the resupply detour.)
2. [S] Watch the strip guard (0x500087) from the landing: does he hold (-7.07,12.24,-218.60)
   in AimStandFire, and does his footwork shift while firing (the port's slide outcome changed
   between trees with his firing)?

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks attempted at ~20:58 UTC (2026-11-10):
- curl (twice, 15 s and 25 s timeouts): `curl: (28) Connection timed out after ... milliseconds`
- python3: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
- wget: exit 0 with no output and no confirmation body — success could not be verified
  (git push to github.com worked throughout, so the block is ntfy.sh-specific).
- A second round at ~21:05 UTC (after the report update): curl timed out again (20 s),
  python3 `Errno 101 Network is unreachable` again, wget failed. The record stands.
