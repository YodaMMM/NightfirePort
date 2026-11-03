# wire-ride-freeze-0c — the wire ride "freeze" is the death sequence: the premise is refuted

Branch `s/wire-ride-freeze-0c`, cut from origin/master (68298698). Task: on 0700000c the #284
wire ride froze byte-exact at d_B 1.52-1.61 (glide 8.12 → 1.61, wire_anim_time stuck 22.6, anim
0x60000b5, auto_drop frozen 20.8, substate 6, sim.disabled false) with "wire_tick stops advancing
mid-ride" (from docs/reports/level-0c-yard-climb-real-health-3.md, "What is left"). Find the
wire_tick branch that stops advancing and port the game's rule.

## What is done

**The freeze is reproduced on this tree, byte-exact, and it is NOT a wire_tick stall — the
rider DIES mid-ride and the port's faithful dead branch freezes the body.** The task's premise
("wire_tick stops executing mid-ride") is refuted; no wire_tick branch hangs.

The repro: the -3's walk runs on master unchanged (0c-local files only — the -3's shared-helper
diffs predate master's newer helper API and were NOT taken), with the west-edge sweep moved to
the LADDER TOP (the -3's measured phase). The sister walk (real damage + `with_yard_aimed_stands`)
freezes at d_B 1.61, mode 0x7, anim 0x0600_00b5 at 22.6, rate 1.2, auto_drop 20.8, sub 6 — the
-3's exact numbers — and the ride assert fails after its 2400-tick window.

The per-tick probe (the ride trace now dumps the gates + health + the death tick every tick
inside d_B 3, NF_RF_TRACE=1) isolates the transition:

- t470-t472: alive, gliding, **hp 1.6** — the rider entered the end zone at 1.6 hp, NOT the
  17.6 the damage ledger prints (the ledger samples at bullet hits; 16 hp of unhooked damage
  landed during the first ~80 ride ticks — the ledger's "lowest 17.6" is its last bullet sample).
- t473: **hp 0.0** — the final pain (≥ 1.6) killed him mid-ride.
- t474: `death.active` true, the death tick 0 — `Player_CheckForDeath` 0x190810 fired [M the
  port's death.rs:216-242 — health ≤ 0 starts the sequence].
- t475+: the death tick advances 1, 2, 3… every tick while the wire state is frozen — the
  death machinery RUNS; the bond tick is skipped by the dead branch [M Player_Update
  0x193600:316-319 — the dead arm runs ONLY AnimObjectUpdate + Player_HandleDeath, no substate
  arm, no collision pass: the body freezes where it died]. `wire_tick` is never called again
  because the substate dispatch never runs — not because any wire branch returned early.

The kill path, by elimination ([L] this session's probes, [M] the cited code):
- NOT a drone bullet — the ledger's 52 hits / 82.4 and lowest 17.6 exclude it.
- NOT a hurt volume — 0700000c has ZERO class-0xE4 placements [L the scratch dump:
  "0700000c hurt volumes: 0 total"; the pass itself is scene.rs ~3428 over Hurt_Update 0x17e5d0].
- NOT drowning (not underwater), NOT fall damage (`fall_damage` is None on the wire — physics
  does not run; it is cleared per tick [M player.rs tick_ctx]).
- NOT a mounted gun (0 active), not the MP paths (SP), not net.
- The BLAST paths are excluded too: the prop dump after the ride shows ONE dead destroy (#3,
  z −132, an early-walk prop) and NO break below 0 anywhere — no prop popped near the gap —
  and the live grenade watch during the ride (any grenade inside the gap box x −20..8,
  z −230..−200) printed NOTHING [L probe 4]. No blast killed the rider.
- What survives: the guard CONTACT/MELEE pain — the melee is not a bullet hit, so the walk's
  ledger never sees it, and the settled walkers (#27/#29, patrol spots −245.6/−252.8) sit at
  the west ledge the rider glides into [S — the next session's first trace: the melee
  dispatch NDrone2's contact arm + a pain-type log at the t473 tick]. The 16 unhooked hp in
  the first ~80 ride ticks is the same question.

So the port's behaviour matches the game's: a rider killed mid-wire hangs dead until the
restart prompt. The freeze is the death sequence wearing the wire's state. "A player must never
be stuck on the wire" holds — the player is dead, and the game shows the death UI; the walk's
2400-tick window just never expected a death mid-ride.

## How it was verified

- The repro walk run 3× on this tree (probe variants): the freeze numbers identical every run
  (d_B 1.61, anim b5 at 22.6, auto 20.8) — deterministic on the fixed input.
- The gate probe: `death true ndead false gimp 0` at every frozen tick, `dtick` advancing —
  the death gate is the ONLY skipped-tick explanation consistent with the frozen auto_drop
  (the only other pauser is mode 0xC, excluded: the trace shows mode 0x7 and a move anim).
- The in-tick contradiction: with wire_tick running, anim time 22.6 + rate 1.2 must advance
  (and the move tree would re-add at the b5/b4 alternation) — the frozen state is UNREACHABLE
  from inside wire_tick, which forced the look above the dispatch.
- The hurt-volume dump (scratch test, not committed): 0 volumes on the level.
- The prop/break dump (in the walk, after the ride window): no near-gap prop died.

The walkreal_*_live sweep and the full suite did NOT run this session: the window went into
the repro + the diagnosis (the release build from a cold cache consumed ~25 min; the sandbox
clock leaves ~30 min at this writing). The changes on the branch are TEST-ONLY (walkreal0c.rs +
the live binary + WORKLOG + this report) — no `crates/*/src` change was made, so the shared
helper/audit sweep is not triggered by this session; the NEXT session's fix will need it.

## What is left

1. **Name the t473 killer.** The blast/grenade paths are EXCLUDED (the probe: no prop died
   near the gap, no grenade entered the gap box all ride). Log the pain call sites at the
   t473 tick (a one-line env trace in pain_generic's callers) — the melee/contact arm is the
   leading suspect. The 16-hp earlier drop (ride t~80) is the same question.
2. **Decide faithfulness at the game level.** If the guards' grenade genuinely lands at the
   ride's hang path in the game, the port is faithful and the "fix" is walk-side: the
   freeze-phase test asserts the death explicitly (the Done criterion "finishes the ride"
   cannot hold while the rider legitimately dies — the ride-finishing proof needs Bond alive,
   which is the -3's remaining ledger: the settled walkers #27/#29's ~70-96 damage).
   If the blast is a port divergence (radius/propagation), fix it with the [M] addresses.
3. Then: the freeze-phase test (repro kept on the branch) adapted to the outcome, the sister
   ride test, every walkreal_*_live, the full suite total line.

The repro stays on the branch: `walkreal0c.rs` carries the freeze phase (the west-edge sweep at
the ladder top) + the per-tick ride trace (NF_RF_TRACE=1) + the kill-source dump after the ride
window. Run the sister with `--ignored` to see the freeze live.

## Live checks for the main agent

1. In PCSX2 on a fresh 0700000c boot, ride wire #284 from the west edge toward B and hover
   near d_B ≈ 1.5-1.7 (the sway zone) for ~30 s with the west-side guards alive: does the game
   ever show the death (the red fade) or freeze the rider? Expected if the diagnosis holds: the
   game either kills the rider (the fade + the restart prompt — a grenade/blast landed) or
   nothing happens (the guards' fire is what differs on the port's tree).
2. fieldprobe: the guards' grenade objects during the ride — does any frag land near
   (−7.4, 12.8, −215.3)? The blast pain is type 0 [M spawn_blast's pain_generic], damage from
   the weapon record.
3. gdbwatch on `Player_HandlePain` 0x1902c8 while a player rides #284 with the west guards
   alive: the pain types observed (a type-0 hit = the blast; the trace's pain-type argument).

## Usage

not visible to me (no token, credit or context-window figures are exposed to this environment;
the only counters visible are process wall-clock times in the tool outputs above).
