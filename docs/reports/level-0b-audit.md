# level-0b-audit — report

Branch `s/level-0b-audit` (from origin/master 30598272). Deliverable: `docs/levels/0700000b-audit.md`
— the per-placement behaviour audit of 0700000b's main map (380 placements [D]), the 16 script
streams (14 in the bin + 2 in the packs) and the mission records, one row each, game behaviour
[M/D] vs the port's, GAP column included. Same three-leg method as the sister audit
(`docs/levels/0700000a-audit.md`).

## What's done

- **The game side, per kind**: the 0x1d04f0 dispatch for all 31 kinds; the four kinds 0700000a
  didn't carry are resolved with addresses — 0xe6 = `Drone_CoverLowNode` 0x137340 (AI cover
  records, NO object), 0x21 = `Ripples_Create` 0x1acc90 (water-ripple emitters, class 0x34),
  0xee = `Trigger_MultiplexOut` 0x1b2ed8 (forces trigger subtype 5, the OUT mirror), 0x2f =
  `Copter_Create` 0x17d340 (p0 ≠ 0 → no object). Class bytes and `control_funcs` rows re-unpacked
  from the raw ELF @ 0x26f890.
- **The disc side, per placement**: `nf-extract placements-all/triggers/nis/level-summary` over
  `0700000b.bin` — every dynamic placement enumerated, the 60-record trigger dump digested, all 16
  streams decoded (the three camera cut-scenes 060004f9/060004fa/0600098a called out; every channel
  write with stream/tick). Method note: the NIS dumps print params in lower-hex — this settled the
  level doc's ch-0x4D vent reading as an erratum (it is ch 0x77, the SP clearing its own out).
- **The mission records**: objectives 0x4C/0x4D/0x4E/0x59/0x5A with every disc writer traced (the
  parachute chain ends OUT #304 → ch 0x5A + 0x64(100) → `LevelToEndTo(0x0700000b) = 0x0730000b`
  [M 0x182c60:99-101]); proven that 0b has NO LOAD_LEVEL placement/trigger/script event — the
  level ends through the mission layer alone [D, all three dumps].
- **The calltree sweep** over the 18 update/control entries: **TOTAL never-mentioned reachable: 1**
  — `Copter_FireMissile` 0x17d9f0, the attack-path row the 08 audit already documented (the Copter
  machine's attack states are unreachable until the mount/spot-check flow ports). Everything else
  is mentioned in the port.
- **The port side, per kind**: created/updated/drawn/usable/reacts/sounds with file:line (§3),
  plus the existing test pins (interactables_0700000b_live, walkreal_0700000b_live — the whole
  route through the real SP machinery — and the census files).

## Fixes this task made (each with a test or an address)

1. **Two unit tests in `crates/nf-game/src/triggers.rs`** (both green; 26 triggers tests pass):
   - `out_mux_mirrors_the_out_channel_both_ways_each_tick` — the OUT mirror law [M 0x1b1a78:147-158]
     in 0b's own #304 shape (out 0x41 → [0x5A, 0x64]); a mirror both ways, not a latch.
   - `the_0b_calc_latch_copies_0x34_into_0x2d` — `Trigger_Calc` 0x1b1900's hard-coded 0b latch.
     This upgrades the port comment's status: it was labelled a "stand-in for #177's unwritten
     gate writer"; the decompile shows it is the game's own level latch (the disc carries no 0x2d
     writer in the 60-record dump [D]).
2. **Four `docs/levels/0700000b.md` rows corrected with addresses** (audit §2): the GT turret row
   said "NOT ported" (stale — gt.rs exists, created drone.rs:12582-12636, fires/dies
   15282-15390, census-pinned); the 0xE6 row said "Hurt … ported (hurt.rs)" (wrong twice: 0xE6 =
   cover records, and 0b has no 0xE4 hurt placements [D]); the vent script's channel write is
   ch 0x77 not ch 0x4D (the lower-hex erratum); ch 0x2D removed from the open-writers list (the
   [M 0x1b1900] latch, now unit-pinned).

## The gaps (audit §5, each with its [M] chain)

1. **GAP-1 (large, code): the keypad E-path** — the 0a audit's GAP-1 applies verbatim to 0b's lock
   #178 (p5 = 47 → the exec-office door #151): `Lock_Activate` 0x1aed00 → the keypad menu
   (MenuManager_Create 0x40000021, the 0x2000014 hint) → state 4 → `Lock_Unlock` 0x1b0650. The
   port has the laser burn (live-pinned) but no E code entry (a menu UI).
2. **GAP-2 (medium, code): the gold-medal kind-6 create gate** [M 0x18dda0:92-96 +
   PlrStats_HasGoldMedal 0x199a40] — 0b carries two BOND_BONUS pickups (#300/#301) that a fresh
   boot would not create. Same cross-level blast radius as 0a's GAP-2 (three live tests + the MP
   medal rule): own task.
3. **GAP-3 (small, determinism deviation): the ripple emitters' RNG** — the game's ripples consume
   the shared `Rand` [M 0x1ac028:31-46]; the port substitutes a counter-hash (world_fx.rs:37-47)
   to protect the shared draw order. Visual-only ([S] — no gameplay channel reads ripples); the
   main agent's render settles the look.
4. **GAP-4 (tracked): the paternoster lift does not carry Bond** (SP #232's cab animates; the
   carry law lives in the SP-entity mount path — the documented walkreal teleport stand-in,
   0b.md:164).
5. **Draw-only (named)**: the census rows 0b(0x27) the GT body + dresser entities, 0b(0x28) the
   sensor camera (a pass exists at scene.rs:8301 — re-count), 0b(0x29) the switch's SP model,
   0b(0xd9) the SP script-entity draws (placement_draw_census_live.rs:157). Meshes: GT
   0x0200064c, cameras 0x02000653.

## Open / live checks for the main agent (audit §6)

1. **The ch 0x54 → 0x08 → 0x4E chain** (objective 9's OR inputs — the one genuinely open channel
   chain): fresh-boot gdbwatch on `switch_channels[0x54]`/`[0x06]` during the files route — what
   writes them (the port completes 0x4E through SP #355's out; the disc's OR #295 inputs are
   unwritten in every dump).
2. The keypad on #178: E → the menu → code → ch 47 → door #151 (GAP-1 confirm).
3. The ripples at the dome puddles: do the RIPPLE_GEN puffs match p0/p1 radius/interval (GAP-3).
4. The paternoster ride on a real pass: does Bond's root ride the cab keys (GAP-4).

## What I ran

- The two new unit tests + the whole triggers module: `cargo test --offline --release -p nf-game
  --lib triggers` — 26 passed / 0 failed.
- The fast runner (`tools/test/suite.py`) detached (setsid): build ok — 811 test binaries in 96 s —
  then the run; the total line and any FAILED/TIMEOUT lines are appended below. No code changed
  except the two tests, so the touched-crates surface is nf-game's triggers module (run above).

## Suite result

(appended at finish)

## Usage

not visible to me
