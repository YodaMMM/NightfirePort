# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## guard-dead-body-shout

2026-11-07 07:28 UTC, branch `s/guard-dead-body-shout` (from origin/master 22d5836b7).

Task: guards react when another guard shouts about a body or a surrender.
react_to_drone_alert_msg in crates/nf-game/src/mp/bots.rs ports the SetState tail for alerts
0x15 (dead body → state 0x9e NDrone2_DSTATE_SeenDeadBody 0x16acd0) and 0x13 (surrender →
state 0x9f NDrone2_DSTATE_SeenSurrenderedDrone 0x16ade0) [M 0x1437a0:267-326;
Surrender_Anim 0x1626a0:74], but only guard_ai_other4_live.rs calls it. Find who raises alert
0x15 and alert 0x13 in the game (every caller) and in the port, and whether the receivers'
gates (props 0x29 and 0x4f, shouting range, +0x4F8 & 0x800000, the +0x50C share, the mode
bytes) are checked on the live path. If a live path already matches, delete the unused copy;
else wire the shared rule into the alert delivery, in the game's order, same Rand-draw count
(0x9f has its Rand(6) bark). List the levels with guards carrying those props [D]. Pin through
real play: a guard finds a body and a nearby guard with the prop goes to the body's alert
position, one out of range does not, and a surrendering guard's shout turns a nearby guard.
Run guard_ai_other4_live, the ai binaries, walkreal_07000001_live, walkreal_07000009_live,
nf-game, and the full suite total line.

- [x] setup: repo cloned, branch created from origin/master 22d5836b7, pushed.
