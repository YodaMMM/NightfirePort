
## results-after-codename-controls
Started 2026-11-05 11:11 UTC. Branch cut from origin/master f68a72216.
Task: the codename flow stops on the wheel — the game forwards to Controls [M 0x213288:38-40]. Port
that step through the real menu path, test with key presses from Main (create codename, reach
Controls, carry on to the mission list). Check Esc on Main against the game (the port returns to the
title and replays the start movie) and fix if it differs.
