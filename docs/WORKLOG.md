# WORKLOG

## player-upward-shot
Started 2026-11-03 01:28 UTC. Branch s/player-upward-shot cut from origin/master 6f16146e.
Task: on 0700000c Bond's aimed rounds at the deck snipers (weapon 0x1e drones ~33 u away and
above him) never land — three clips dry, target hp unchanged, strict gun rays true, while level
shots at the same range work. Trace one upward shot through the port's bullet path (flight
segment, per-tick HITTEST, Collide_Pick straddle cels, lip geometry), compare each step with the
game [M Bullet_update 0x12e7b0, Collide_Pick 0x1eb320, cel straddle 0x1eac98], fix any place the
port stops a round the game lets through, and pin the outcome with a real-input test from the
yard firing up at a deck sniper.

Work log:
- (01:28) branch start, WORKLOG entry.
