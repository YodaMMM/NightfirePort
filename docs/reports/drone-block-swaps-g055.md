# drone-block-swaps-g055 — the other behaviour-block swaps (GAPS G-055)

Branch `s/drone-block-swaps-g055` from origin/master c3e3135f. The three G-055 swap sites are
wired, each with its decompile address, unit pins with sisters, and a real-load live pin.

## What's done

1. **`FirstAttack`'s alternate-state branch** [M 0x148010:100-131] — in the ATTACK-ENTER generic
   arm, after a zero first-sight pick and on FirstAttack's own pass only (re-entries return 0 at
   `+0x4F8 & 0x2000000` [M :17-19]), the drone with `+0x5A4 != 0` runs the ChangeToAttackMode
   flush inline: `+0x5A2 = 0x56` [M :102], `+0xC6 → +0xC5`, the mode-2 sniper retype, both
   cleared, and with `+0xC48 == 100` the SECOND block comes in [M :118-128]; the return is the
   old alt state with the 0xb4→0x56 remap that is FirstAttack's own [M :128-131].
   (`+0x135` is provably clear at that point — the common path just cleared it and neither
   `DroneAlertToObject` nor `AttackTalk` writes +0x135 [M the +0x135 writer census].)
2. **`PreDroneControl`'s second-switch flush** [M 0x148db0:58-82] — the channel-ON flush now
   runs the ChangeToAttackMode inline when `+0x5A4 != 0` (mode swap, sniper retype, the p5==100
   Set2nd, the verbatim SetState(alt)); the verbatim-0 arm is kept. The old code passed `+0x5A4`
   through with no mode swap and no block switch.
3. **`HandleImpact` case 7's 09/0A stamp arm** [M 0x1469b8:103-128] — on 0x07000009/0x0700000a
   the fall tase (accumulator past 2·health, [M :72-88]) clears the first-attack/sight latches,
   arms `+0x5A4 = 0x89` RunToAlarm, `+0x16 = 0`, `+0x4FC = 0`, restores the FIRST block when
   `+0xC48 == 100` (the Set1st), sets `+0xC5 = 9`, `+0xC6 = 7`, `+0x5A2 = 0x14`,
   `SMI+0x10 (= +0x118) = 0x15`; the latch-clear arm writes only the `0x14` resume [M :105-106].
   The three identical blocks in the smoke/flash/dart cases (0x1469b8:220-246, 285-310, 331-356)
   are NOT wired — open, listed below.

## How it was verified

- **Unit pins** (crates/nf-game/src/drone.rs `#[cfg(test)]`, each with its condition-false
  sister): `second_switch_flush_swaps_in_the_second_block`, `first_attack_alt_tail_swaps_in_the_
  second_block`, `tase_fall_on_090a_swaps_back_to_the_first_block` (level gate, latch gate).
- **Live pins** (crates/nf-view/tests/drone_block_swaps_live.rs, real level loads, real input/
  message paths):
  - Flush: 07000003 wake guard #388 (p6=80, p8=100, p5=100 [D]) — raising ch 80 swaps the
    second block in, consumes tag/channel, `+0xC5` 0x5→0x7; the ch-79 sister #389 is untouched;
    120 ticks channel-OFF change nothing.
  - FirstAttack tail: 07000003 #400 (enabled, alt 0x56 [L the probe]) — Bond placed in front on
    the real sight path until the first attack: `using_2nd` true, `+0x5A4` consumed; the
    no-alt-state sister keeps its first block.
  - 09/0A stamp: 07000009's p5=100 alarm civilians [D #62/64/65/66/250/261] — the fall tase
    leaves state Taser 0x4a with ret 0x14 / resume 0x15 / mode 9 / alt 0x89 / latches clear;
    the unsighted sister gets only resume 0x14 with alt/return untouched.
- **Runs** (this sandbox kills background processes between tool calls, so the fast suite
  runner could not be used; foreground slices instead): nf-game lib **658 passed / 0 failed /
  5 ignored**; nf-game integration all green (ai_live 37, stealth_ai_live 9, the rest ok);
  nf-formats **24 ok + disc_golden 11 ok / 3 failed — exactly the 3 `disc_golden` failures the
  slim ISO allows**; nf-audio all green; nf-view: drone_block_swaps_live 3/3, walkreal_
  07000001_live 6, walkreal_07000002_live 2, walkreal_07000003_live 5, walkreal_07000006_live 5,
  walkreal_07000007_live 3 — **all passed, 0 failed**.

## What's left

- The full suite run (tools/test/suite.py) and the remaining ~73 walkreal_*_live binaries did
  not fit the sandbox window (the sandbox also filled its 9.9 GB disk mid-run; ~1.9 GB was
  freed and the three-crate run above completed after that). No walk failed; the unrun set is
  a risk to retire next session, first walkreal_07000005/09/0a/11/12/13 (the alt-state and
  corridor levels).
- The HandleImpact twin sites (smoke 0x1469b8:220-246, flash :285-310, dart :331-356) are still
  unwired — same block, different cases; wire and pin them the same way (the 09/0A census shows
  p5=100 guards exist on 09/0a, so the swap arm is reachable there).
- Live checks for the main agent: during a 07000009 play session with the Stunner, gdbwatch
  `Drone+0x5A4` (expect 0x89 after a tase fall on a sighted civilian), `+0x5A2` (0x14), `+0x118`
  (0x15), and `+0x29`/`+0x4D8` (the Set1st — expect `+0x4D8` back at `+0x4DC` when it started at
  `+0x4E8`); and on 07000003, raise the ch-80 switch and watch `+0x29` go 1 with `+0x4D8` at
  `+0x4E8` and `+0xC5` = 7.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no numbers were visible in any interface output to copy).
