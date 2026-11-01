
## drake-round-lifetime — 2026-10-31

Task: on 0700001b, Drake's weapon-0x30 rounds fly slowly and stay live 600+ ticks; one hit Bond
for 12 during the defeat cut-scene after Drake was already dead (found while making the real-health
Equinox walk pass, origin/s/1b-real-health). Check that weapon's projectile speed, range/lifetime,
what happens to a dead shooter's live rounds, and damage during a cut-scene (player disabled /
invulnerable?) against the game [M Bullet_update 0x12e7b0, bullet lifetime/range fields, cut-scene
player-disable path; D weapon 0x30's data]. Fix divergence, pin with a test + sister. Continue from
origin/s/1b-real-health (origin/master merged). Run walkreal_0700001b_live, interactables_0700001b_live,
drake_fight, nf-game and quote every total.

Progress (drake-round-lifetime, 2026-11-01): sandbox reset wiped the tree at 01:38 UTC; re-cloned,
branch resumed from the pushed checkpoint. Investigation: the four flight facts all check against
the game — speed = record +0x20 ×0.25 (Bullet_init 0x12c5a0:188-191), range = +0x1C (Bullet_update
0x12e95c-0x12e984 + the boundary clamp 0x12f00c-0x12f02c), no owner gate in the flight, and
Player_HandlePain 0x1902c8:14-26 has no cut-scene arm (the only invuln gate is switch_channels[0x5f]).
One divergence found and fixed: Bullet_DoTrails' 0x200-ramp (0x12d924-0x12d964) runs on the SPAWN
tick too (Bullet_update state 0 falls into the same DoTrails tail, disasm 0x12efd8-0x12f000) — the
port's `born != now` filter skipped it; Drake's 0x30 rounds now leave the muzzle already ramping.
New test drake_round_lifetime_live (nf-view): record values [L ee_start_05 @0x2bf150+0x30·0x10C],
spawn-tick ramp, 38-tick overshoot cap 0.11783366, 600-tick survival with the shooter gone, range
death at 500. Known gap: Drake's AI never fires at an idle Bond in the port (astronaut_move_pick's
`firing = los` never opens — visibility < 0.7 at 16600), so the cut-scene 12-damage hit is pinned
by the code citations, not by a sim run; live check requested in the report.
