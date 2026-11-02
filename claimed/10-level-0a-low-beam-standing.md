# level-0a-low-beam-standing — the low beam #621 fires on a standing player

Branch from origin/master (it already carries s/player-trigger-contact).

`walkreal_0700000a_live::low_beam_621_fires_standing` is ignored as a port gap: the standing trigger toucher is the box pos−0.05..pos+1.26 r 0.55 (drone.rs player_touchers), and its reach does not meet the low beam #621 the way the game's player body does.

Do:
1. Re-run the test on master first: player-trigger-contact may already close it. If it passes, un-ignore it and stop there.
2. Otherwise derive the player's trigger contact body from the game [M Player_Collision 0x194df0 and the trigger contact pass] and fix drone.rs's toucher to match; cite every number.
3. Un-ignore the test; run walkreal_0700000a_live, interactables_0700000a_live, every interactables_*_live, then the full suite.

---
claimed-by: sandbox4 at 2026-11-02T05:18Z
