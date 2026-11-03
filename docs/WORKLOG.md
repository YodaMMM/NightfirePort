# Worker WORKLOG (per-branch; master strips this file)

## satchel-laser-scramble (started 2026-11-02 ~23:26 UTC)

Branch cut from origin/master 0ea03862. Task: on 0700004a, lasering the placed satchel charge
must (a) spawn the game's spark particles where the beam hits and (b) after a short exposure
scramble the countdown digits. Plan: find the satchel-vs-laser path in the decomp [M] (satchel
update, laser-hit handling, digit scramble state + timer, spark emitter), port both through the
existing particle/UI systems, add a real-input test (place satchel, laser it, assert scramble
state + spark emitter), then satchel/laser tests, interactables_0700004a_live, full suite.

satchel-laser-scramble (final, ~01:16 UTC): the player's placed satchel burns under the watch.
Root cause: the fired satchel rides the thrown-grenade pool, which the 0x4E/0x4F burn walk never
scanned, and Grenade had no BU+0x102/burned-out state. Grenade gains burn+defused; the 0x3b tail
takes the digits before the burn arm, redraws the fuse to Rand_Rand(0x1518)+0x3c every tick, skips
the beep, never explodes; the walk covers the grenade pool. Real-input 4a test green (ride, both
shooter charges burned, park, lobby scene skipped, place, laser: sparks/100 ticks/0x5dd/scramble/
never blows). nf-game lib 656 ok; nf-formats+nf-audio green except the 3 allowed disc_golden; the
satchel/laser/interactables/walkreal 4a batch + sound_coverage all green. The full suite did NOT
fit the window (the 27 test-literal rebuild was still running at 01:05) — the next session runs it
from this branch. Report docs/reports/satchel-laser-scramble.md.
