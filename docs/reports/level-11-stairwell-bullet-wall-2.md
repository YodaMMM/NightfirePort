# level-11-stairwell-bullet-wall-2 — bullets pass the stairwell fudge without breaking other shots

Branch `s/level-11-stairwell-bullet-wall-2` (from origin/s/level-11-stairwell-bullet-wall
53dca022, merged with origin/master 34ac31a5 → 1ce7edf7; fixes through 13d751ca).
Window 10:20–12:20 UTC.

## What's done

**The gondola window (08) — root-caused and fixed [M].** The stairwell flight change
(`segment_bullet` for the flying bullet's world trace) did NOT make the round pass the window
it should break — the window panes have no triangles in the collision sets at all (the 16 pane
glists 0x0200037d..0x0200038x resolve to no named object; probe
`bullet_pass_probe.rs::dump_gondola_pane_materials`). The red was the OTHER half of the same
tick: the round died on the pane (`b.dead = true` in the pane-hit block), so the post-kill
flight segments the test counted were gone. The game's rule is drain-WITHOUT-stop for glass:

- `Collide_Update` 0x1ea398's walk (the 216-229 region) passes a class-0x10 surface whenever
  the bullet's HITTEST+0x9a carries the 0x20 mask [M disasm 0x1ea6cc-0x1ea6e4: `(mask 0x20 &&
  class != 0x10) → the computed pass; class == 0x10 → pass`], and `Bullet_update` 0x12e7b0
  arms that mask for every gun without weapon_data+0x6c & 0x800 [M disasm 0x12f048-0x12f058];
- `Bullet_CollisionHandler` 0x12ce80 has the same glass arm under the same +0x6c & 0x800 gate
  (`if ((HITDATA+0x50 & 0x3f) == 0x10) pass = 1`), after the class-'(' breakable stop;
- the pane still takes the damage: the collision pass appends the contact to the hit object's
  obj+0xD0 chain whether or not the round stops (the same pass the fusebox wires and
  `SS_Operate` 0x1a9df8 read), and `Break_Update` 0x12bfd8:36-37 drains health from it [M].

The fix (nf-game/src/drone.rs, the pane-hit block): a kind-2 (glass [D params; M 0x12c230:21])
pane's hit drains health and can `Break_Kill` WITHOUT killing the round; a non-glass
breakable keeps the stop. The old test pin (`bullet_paths` non-empty 7 ticks after the kill)
held on master only through the accidental double-pierce of the pass-flagged glass static in
front of the pane (the pre-fix `world.segment` saw it, the flush's penetration roll let the
round through); re-anchored to the kill tick (`paths_at_kill > 0`), which is the actual
"the kill came from the bullet pass" claim, with the [M] chain in the comment.

**The 01 escort (escort_follow_07000001_live) — green.** Re-run on the fixed tree: 1/1. The
fight resolves differently now that Bond's rounds pass the pass-flagged polys (01 carries
`EntryHall_a`, `gallery_blockingpoly` and friends with 0x40-carrying material bytes [D
`dump_07000001_shoot_targets`]) — the endgame landed differently and the test passed as-is
after the gondola fix.

**The stairwell + foliage pins — green.** stairwell_bullets_07000011_live 2/2,
fix_foliage_bullets_live 3/3 — the glass-pass change does not touch the stairwell's mat-0x40
fudge walk or the foliage core/leaf rules.

**nf-game lib 547/547** (the NIS/bullet/ally core), **zoe_gondola_realpath_live 1/1**,
**gondola_windows_live 3/3**, **escort 1/1** on the final tree.

## What's left

**walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact — still red** (the
third listed test). The fight's new resolution parks the meet-hall contact in
AllyLeadWait (0x20) at route node 15 (6.9, 6.3, 9.1) with Bond held 6.1 away — the port's
release [M 0x15eda8 case 3, the port's ALLY_LEAD_WAIT arm: `pd > -7.0 && (pd != 0.0 ||
dist2d < 5.0)`] reads `pd == 0.0` (Bond off the route's 5.0 node scan) and needs him within
5.0 straight-line, but Bond's own walk stalls at (1.38, 6.7, 11.61) in the hall (the nav gives
up at its 3000-tick cap; the trace is in the session log). The harness changes pushed (the
stall cap 24, the 1.2 close, the vault assist on a failed walk-past) got the run past the
earlier stalls to the final route check (`the contact must walk his route to the plate`); the
next session should trace what body/geometry holds Bond at (1.38, 11.61) — the candidates are
the meet scene's frozen guards parked mid-lane (the fight now leaves one at (7.0, 6.3, 8.5),
state 0x1, enabled false) — and play the lane the player's way (around or vault), or re-tune
the meet-phase choreography.

**The full four-crate suite** did not fit the window (the background runner is killed between
tool calls in this sandbox; the foreground run exceeds the remaining time). What ran on the
final tree: nf-game lib 547/547; gondola_windows_live 3/3; stairwell_bullets_07000011_live
2/2; fix_foliage_bullets_live 3/3; escort_follow_07000001_live 1/1;
zoe_gondola_realpath_live 1/1. The next window should run the full suite and report the total
line.

## Live checks for the main agent

1. **The gondola window's break vs the round's life**: fresh boot of 07000008, ride the cable
   car, break a window with a gun (gdbwatch on the shooting round's BU+0xF4 state after the
   pane contact): the game's round should keep its state-1 flight (or die on the far wall) —
   NOT stop inside the pane's bounds. A screenshot mid-burst should show the far-side wall
   sparks behind the breaking window.
2. **The meet-hall lane (07000001)**: fresh boot, play to the meet scene's end, then walk
   Bond from the hall's east side toward the contact standing near (6.9, ~6.3, 9.1) — is the
   lane clear for the player (no invisible/frozen body at (1.4, 6.7, 11.6))? Which bodies
   stand in the hall after the meet, and do they block the player's walk (the port's Bond is
   held 6.1 away with the nav giving up)?

## Usage

not visible to me — this session's environment shows no token, credit or context-window
figures (no usage display was available at report time; nothing estimated).
