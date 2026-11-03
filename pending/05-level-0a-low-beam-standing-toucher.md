# level-0a-low-beam-standing-toucher — the 0a low laser beam must trip when Bond walks through it standing

Measured in the original [L PCSX2 2026-11-02, fresh goto_level 0700000a]: Bond placed STANDING at (-12.4, 0.608, -31.0), yaw pi, walked forward 70 frames to z -33.78 across beam #621's line: ch 0x24 goes 0 -> 1. The game's standing body trips the low beam.

The port does not: walkreal_0700000a_live::low_beam_621_fires_standing is ignored with "port gap: the standing trigger toucher is the measured box pos-0.05..pos+1.26 r 0.55 (drone.rs player_touchers) and misses #621's band y 0.13-0...". Find the game's player-vs-trigger toucher shape [M Collide_PickObj 0x1eae80 and its HITTEST type word, the player's contact list build] and port it, so the standing walk-through fires ch 0x24 while every other trigger test stays right (the 0a crouch-under cases, the 05 truck plate, the fix_truck sisters: re-check each against the same law and fix the wrong ones with the address).

Un-ignore low_beam_621_fires_standing. Done = it is green, every trigger/beam test green, every walkreal_*_live green, then the full suite total line. List each binary you ran.
