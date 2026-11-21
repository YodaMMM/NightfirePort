# s-crouch-latch: Bond's crouch as ACTION.ELF does it

No shared random stream change by construction (no draw added or removed). Walks that press
crouch at new times now take different paths; see the reds below.

## What the game does [M unless tagged]

- Crouch: `Player_HandleJump` 0x1916b0:89-95, walk only, jump state 0: the crouch press EDGE
  (`Input_Action(p, 8, 4)`) with `BL+0x960` clear → `ChangeSubState(4)`. Both Crouch options use
  the edge. A press inside the lockout is lost; nothing buffers it.
- Stand: `Player_SSCrouch` 0x1924c0:84-105. Toggle = press edge of 8 or 7 (jump). Hold = action 8
  not held. Also needs the head-bump bit clear, the lockout clear and the 0.95 headroom line.
  The crouch handler never calls `Player_HandleJump`, so a jump press only stands Bond.
- Lockout: `ChangeSubState` 0x190630:42-56 sets `+0x960 = FRAME_RATE` (60) on 0→4 and 4→0 and
  plays the transition script (SP class 3: 0x060001dc / 0x060001dd, speed 4). Both scripts are
  50 frames, non-looping on every level in `dev::LEVELS` except 07000048, which has neither [D].
  `SSWalk`/`SSCrouch` count the lockout down and zero it once `AnimScriptIsStopped` sees the
  script past its end, or at 1. The script stops on its 13th clock tick, so **the next stance
  press works 13 ticks after a change** (59 is the fallback). Swim and zero-g do not count it.
- Other exits: any `ChangeSubState` out of 4 ends the crouch. `ChangeSubState(0)` from 4 (the
  cut-scene hand-back `Player_Cam2Mode` 0x195fc8) stands Bond with the lockout. Ladder, wire,
  zipline, grapple, creep and swim entries end it with no lockout, and those substates hand back
  a standing walk.

## What changed in the port

- `Bond`: the lockout ends with the 50-frame transition clock (`stance_clock`/`stance_stopped`)
  instead of a flat 60, and is frozen in swim/zero-g. `change_substate` drops the crouch: stand
  plus lockout into the walk, no lockout into any other substate. Before, the flag survived
  ladders, wires, the claw and swimming, so Bond came off a ladder still crouched.
- `Input::crouch_held` (new, local only): Hold mode stands on `!crouch_held`. The keyboard now
  feeds the press edge into `crouch` in both modes. Before, Hold fed the held level into the walk's
  crouch test, so a key held through the post-stand lockout crouched Bond again.
- What a player notices: crouch/stand toggles respond after about 0.2 s instead of a full
  second; climbing a ladder, riding a wire or firing the claw from a crouch leaves Bond
  standing, as in the game; a cut-scene that hands control back stands him up.
- Tests (all three fail on the old code, checked by putting the old `player.rs` back):
  `the_stance_lockout_ends_with_the_transition_script`,
  `the_hold_option_crouches_on_the_edge_and_stands_on_release`, `a_substate_change_leaves_the_crouch`.
- docs/systems/player.md: the Crouch section rewritten.

## The 07000008 00 Agent walk (ignored)

The walk's question is now answered: the latch was the port's flat 60-tick lockout. The walk
presses crouch at the hole, waits for the fall (about 16 ticks), then presses again. On master
that second press landed inside the 60 ticks and was lost, so the fight always ran crouched
(death t21070). With the game's lockout the press stands Bond, and both stands I measured (at the
13.2 break and after waiting for a grounded tick) lift him back onto the upper floor at
y 13.4704 in the zap band (deaths t21064 / t20944). So standing on the lower floor was not
reached. I removed the second press, which keeps the shipped crouched fight exactly as it was
(t21070 reproduced). The test stays ignored.

## Test runs (branch vs base 82765b537, same lane)

- Green: player unit tests (nf-game lib), nf-view lib `the_crouch_option_picks_the_stand_condition`,
  fix_0c_hatch, fix_0c_vent2, fix_hud, fix_swim, hatch_crouch, hatch_crouch_real, hud_playtest,
  player_moves, stealth_ai, walkreal 03, 05, 06, 07, 08, 0a, 0b, 0c, 0c_ventexit, 11, 14, 15, 16.
- Fixed tests: camera_death_crouch, interactables_07000012 and pane_solid_06 pressed crouch on
  every tick, which under the old lockout only toggled every 60 ticks. They now press once, or
  every 60 ticks for the pane route that needs the stand/crouch alternation. All green.
- walkreal_07000013 `..._real_damage_agent`: red in the whole-binary run on BOTH base and branch,
  green alone on both, with identical logs. Not caused by this branch.
- **RED, caused by this branch: walkreal_0700000d `walk_arrival_to_the_c5_door_real_damage` and
  `..._agent`** (base green). Traced: the walks relied on the old bugs. (1) d1's deck-crouch
  presses were always lost on base: the jump-grab's stand armed the 60-tick lockout and the wire
  ride froze it. Now they crouch Bond, and the deck walk takes +404 ticks instead of +281. (2) Both
  difficulties carried the crouch flag through the claw hang (substate 2) and onto the gantry, so
  the ride ran with the crouched capsule. A fresh press aboard brought d1 back to green, but d2
  still diverges because the hang lands at a different height without the crouched capsule. The
  0d walk needs a retune on the game's rules. I did not change that walk on this branch.
- walkreal_07000001: the other tests are green. `following_mayhew_down_completes_the_level` hits
  the 590 s cap on BOTH base and branch, run side by side on the shared machine, so there is no
  verdict for it.

## Unproven

- The 13-tick figure assumes `AnimObjectUpdate` advances the new script the tick it is added
  (the order `mp::body_anim` already uses). Not live-checked.
- A weapon change during the lockout rebuilds the anim set (`Player_SetWeaponAnim` →
  `PlayerAnimSetInit`), replacing the transition script. The port does not model this.
- What a missing script (07000048) does to the lockout. The port keeps the 50-frame clock there.
- The crouched root height still snaps to 0.55 instead of following the script's root track
  (motion, left alone).
- The gamepad's Hold mode does not feed `crouch_held` (gamepad work is off limits), so a pad in
  Hold mode stands Bond once the lockout ends. Before, it stood him after 60 ticks.
