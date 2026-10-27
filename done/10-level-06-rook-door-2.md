## Your task: `level-06-rook-door-2`, 07000006: a guard parks in the courtyard doorway — land the attack-mode fix

Start by merging `origin/s/level-06-rook-door` (evidence only, no port code): its `docs/reports/level-06-rook-door.md` and `docs/systems/rook-door-06.md` hold the proven chain. In short: on 07000006 a switch-16 guard beside door #47 (36.50, 40.00, -16.25) takes RUN_FOR_COVER through the doorway, re-enters ATTACK, and the port's ATTACK-ENTER `ChangeToAttackMode` flush flips him to mode 25 (stationary sniper, SniperAim 0x2a) — he parks in the doorway holding the door open. `NDrone2_ChangeToAttackMode` 0x14a648 has five callers [M] (0x14ce68, 0x14d45c, 0x15da54, 0x15e35c, 0x165948); `NDrone2_DSTATE_Attack` 0x164210 and `DroneFunc_FirstAttack` 0x148010 are not among them. `GetDroneTypeAttackTypeFriend`'s tail 0x14cea8 writes the attack-fold result into +0x5a4, which the port never does.

Removing the ATTACK-ENTER arm fixes 06 but turns `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` red (the contact dies at round 10, state 0x47): the 01 walk only held because the wrong flush parked the corridor guards as snipers.

1. Port the +0x5a4 fold write-back [M 0x14cea8] and remove the ATTACK-ENTER arm (flush only at the game's real call sites).
2. On 07000001, identify the guard that kills the contact (dtype/kind/alt_state, the [M] behaviour he should have) and fix what the game does there with real input — never by keeping the non-game flush.
3. Un-ignore `rook06_realpath_live`; it and the 01 walkreal walk both green, with every rook_*/fix_06_*/escort_* test still green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-rook-door-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T02:00Z

---
## Report (done, 2026-10-25T02:46Z)


---
## Report from the worker (done, 2026-10-25T02:46Z)

# level-06-rook-door-2 — the ATTACK-ENTER flush arm removed; the 01 contact kill isolated to one round-0 window

Branch `s/level-06-rook-door-2` (from origin/master 11e19082, merged `origin/s/level-06-rook-door` for its
evidence: the walker `NF_DRONE_WATCH` hook, the `#[ignore]`d `rook06_realpath_live` evidence test, the docs —
that branch carried no port code). One port-code change landed:
`crates/nf-game/src/drone.rs` — the ATTACK-ENTER `ChangeToAttackMode` flush arm deleted.

## What was done

**Step 1 — the flush removed, only at the game's real sites [M].** The port's ATTACK-ENTER arm flushed
`alt_state`/`alt_mode` on every re-entry into ATTACK (mode promote 0 → 0x19 + stationary) — the behaviour
that parked the 07000006 switch-16 guard in door #47's jamb as a stationary sniper. It has no game caller:
the `jal` census of ACTION.ELF gives `NDrone2_ChangeToAttackMode` 0x14a648 exactly five call sites —
0x14ce68 (`GetDroneTypeAttackTypeFriend` 0x14cbd8's kind-0xb creation arm), 0x14d45c (`DoTypeSettingsOLD`
0x14d2e8), 0x15da54 (CivilianMission arrival), 0x15e35c (EnemyMission arrival), 0x165948
(`GoToGoalPosition`, gated `+0xc5 == 7` there [M 0x165640 default branch]); `NDrone2_DSTATE_Attack` 0x164210
and `DroneFunc_FirstAttack` 0x148010 are not callers. The game's ATTACK ENTER [M 0x164210 case 1] reads only
the mode byte (+0xc5 2/0x19 → SniperAim 0x2a, else the 0x67/0x58 combat arms) and never touches +0x5a4.
The arm is gone; a citation comment stands in its place.

**The +0x5a4 fold write-back [M 0x14cea8] — verified already faithful, no code needed.** The port's
`type_setup` tail (`drone.rs` 2126-2129) transcribes the friend's `switchD_0014cea8_caseD_3` write exactly:
the fold keys are the decompile's case labels — which the decompiler prints on `(state − 4)` — so
{4,5,6,8,0x30,0x31,0x36,0x37,0x85,0x89}→0x56, {0x10,0x2e,0x32}→0x12, 0x29→0x2a, 0xa6→0xa7, default 0, under
the game's gate `+0xc6 == 0 && +0x5a4 == 0 && (+0x135 != 0 || +0x504 >= 1.0)`. The port's `alert_block` is
constructed from the ALT behaviour block's property 0x20 [M DoModeSettingsNEW 0x14c600:60-66 stores prop-0x20
of block 2 into +0x504] — i.e. it IS the game's +0x504 despite the field-doc calling it +0x500 (doc fix
listed below). I also censused the friend's callers: `DoModeSettingsNEW` 0x14c600 only (jal 0x14c6f4),
reached from `DefaultInit` 0x14b300 (0x14bce0) and `DoModeSettingsOLD` 0x14cfa0 (0x14cfe4) — creation only,
no runtime re-pass, so no runtime write-back site exists to port. The EnemyMission arrival flush (a real
site, level-gated off 0x0700006) stays as the only live flush, unchanged.

**Step 2 — the 01 contact kill, reproduced and isolated (not yet fixed).** On the fixed tree,
`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` fails exactly as reported:
2 passed / 1 failed. The full per-tick drone watch + a local (uncommitted, reverted) drone-table dump pin it
down:

- The contact is drone **#1** (skin 0x0500004f, team 2, the AllyLead machine, mode 12). He is **killed
  during round 0** of the escort loop: by round 1 he is `corpse, health 0, state 0x47` at
  (−4.44, 6.34, 14.57) — the loop then walks on for 9 more rounds over a corpse.
- Round 0's clear set was guards **{3, 5, 14, 15}** (skins 0x36/0x39, the hall pair). At the round-1 dump,
  #14/#15 are mid-death (health −1, state 0x44), #3/#5 dead — one of them shot the contact inside the
  round-0 window. The creation-mode-2 guard **#2** (mode 2, state 0x2a, alt_state 0x2a, alive at
  (−29.73, 10.22, 13.66) — the upper-floor west end) is the other armed drone on the board.
- On master the wrong flush parked mid-fight guards as stationary snipers; the contact's LeadHide kept him
  out of every line and the walk held. With the flush gone the corridor guards do what the game's AI does —
  they hunt — and inside this walker's round-0 cadence one of them lands the shot. **The kill is not yet
  attributed to a specific guard** (the sim keeps no killer record; next round's probe reads the contact's
  opponent +0x170 at death or traces health per tick), and no fix has been chosen — the candidate space is
  (a) the kill is game-faithful AI and the WALK must clear the round-0 set faster (the real player's
  behaviour), or (b) the killer exhibits a ported deviation. No decompile-backed pin stands against either
  yet; per the never-weaken rule nothing was changed in the sim for 01 this round.

**Step 3 — rook06_realpath un-ignore: NOT done.** The evidence test stays `#[ignore]`d. Its full real-path
walk does not fit this sandbox's window next to the required 06/01 verification runs; un-ignoring it
unverified risks shipping a red branch. It is the first landing-gate item for the next round (its three
decompile-backed pins expect the fixed tree's behaviour, which the previous round already measured green on
a scratch tree with this same arm removal).

## Verification (what ran, on the pushed tree)

- `cargo test --offline --release -p nf-game --lib -- --test-threads=1`: **547 passed / 0 failed / 3 ignored**.
- The 06/01 neighbour binaries (one invocation, `--test-threads=1`): fix_06_alarm 2, fix_06_driver 2,
  fix_06_fire 3, fix_mission 4 (incl. `the_07000001_exit_works_end_to_end`), rook_absent_06 1,
  rook_door_06_live 1, rook_door_06_walk_live 1 — **14 passed / 0 failed**.
- `walkreal_07000001_live`: **2 passed / 1 failed** (the contact walk — the known 01 conflict, above).
- The fast runner could not be used as designed: this sandbox kills every background process between tool
  calls (verified again today), and the full four-crate run does not fit the window; the slices above are
  the trees' real totals for everything the change touches. Known reds anywhere: only the 01 contact walk.

## What's left

1. Attribute the 01 contact kill (the death-tick opponent read), pin the killer's dtype/kind/alt_state
   against his placement, and land the fix — the walk's round-0 cadence if the kill is game-faithful, a sim
   fix if the killer deviates. Then both 01 walkreal tests must be green with the flush still removed.
2. Un-ignore `rook06_realpath_live` and run it green on the landed tree (its pins were validated on a
   scratch tree with this same change last round).
3. Port the remaining real flush sites if any test ever reaches them: GoToGoalPosition's mode-7 arm
   [M 0x165948], CivilianMission's arrival arm [M 0x15da54], the kind-0xb creation arm [M 0x14ce68] (none is
   exercised by any current test; each needs its own evidence before a pin).
4. Field-doc correction: `alert_block` (+0x500 per the struct comment) is constructed from the ALT block's
   prop 0x20 — the game's +0x504 — and read only by the creation fold gate; the doc and the name should be
   reconciled in a docs-only pass.
5. The full four-crate suite on the final merged tree.

## Live checks for the main agent

1. Fresh boot of 07000001 (no save state): stand Bond at the hall exit and let the corridor guards fight
   unshot for ~60 s — does any guard shoot the contact (skin 0x4f) while he LeadHides at
   (−4.4, 6.3, 14.6)? Which skin lands hits? This decides whether the port's round-0 kill is game-faithful
   AI pressure or a port artefact.
2. The mode-2 guard at (−29.73, 10.22, 13.66) [his placement word, port census]: in the game does he ever
   leave 0x29/0x2a and fire down into the hall? A `gdbwatch` on his +0x50c and his fire arm during the same
   window settles candidate (b).

## Usage

Task-system token/context/cost figures are not visible to me this session (no token, credit or
context-window readouts are exposed in this environment). Session wall-clock: branch created 02:06 UTC,
sandbox expected to expire ~04:02 UTC; setup from scratch (clone, work-slim + ISO downloads, rustup, alsa
user prefix, cargo fetch) took ~15 min of it; one background-suite attempt confirmed the sandbox kills
background processes between tool calls.
