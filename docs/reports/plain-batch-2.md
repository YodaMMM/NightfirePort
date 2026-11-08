# s/plain-batch-2

Merge batch on master 7876b7feb. Each branch has a line-by-line review in the local review folder.

Merged: xray-only-breakables, blast-cookoff-2, control-funcs-update (its code arrived identical through
blast-cookoff-2; squash left nothing new), agent-capsule-push, agent-gadgets, agent-pickups,
agent-player-moves, agent-crouch-2, agent-qworm-ray, agent-recovery-3 (squashed), agent-keypad-2,
agent-music-merged (music.rs taken from the branch), agent-lvl-15-16-b, agent-lvl-12-13-b,
agent-cable08, unread-params-props.

Held: agent-1b-scripts (it ignores a 1b walk that passes on master and makes undrawn script
entities solid).

Conflicts: scene.rs camera gate + Q-Worm gate kept both (the camera check first, as
Player_WeaponFiring orders them); keypad drone.rs/lock.rs took master's key-ring path
(Lock::key_tick, equip_pending), the branch's direct equip dropped.

Test fixes after the merge:
- keypad_entry_live: the sister keypad opens from the ring with no press (Lock_Update case 0,
  0x1ae3d8:139-147), as master's key_tick models it; the branch test expected a press.
- interactables_0700000a_live: teleport (now place_bond) waits for the root to settle before the
  Q-Worm aims; the +1.1 stand sank 0.1 during the aim and the thin PC mesh was missed.
- capsule_push_07_doorway_live: at the party's 0.9 pace the nearer guard stands between Bond and
  the west jamb, so the press uses try_walk_to; the floor/grounded asserts are unchanged.
  Unproven: with the batch's timing neither this test nor flank_fall_repro goes red without
  clip_push_to_world, so the clip has no discriminating test here.
