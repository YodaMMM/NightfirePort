
## drake-round-lifetime — 2026-10-31

Task: on 0700001b, Drake's weapon-0x30 rounds fly slowly and stay live 600+ ticks; one hit Bond
for 12 during the defeat cut-scene after Drake was already dead (found while making the real-health
Equinox walk pass, origin/s/1b-real-health). Check that weapon's projectile speed, range/lifetime,
what happens to a dead shooter's live rounds, and damage during a cut-scene (player disabled /
invulnerable?) against the game [M Bullet_update 0x12e7b0, bullet lifetime/range fields, cut-scene
player-disable path; D weapon 0x30's data]. Fix divergence, pin with a test + sister. Continue from
origin/s/1b-real-health (origin/master merged). Run walkreal_0700001b_live, interactables_0700001b_live,
drake_fight, nf-game and quote every total.
