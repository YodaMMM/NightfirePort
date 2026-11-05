# WORKLOG

## fx-scrape-and-impacts
Started 2026-11-05 00:52 UTC. Branch s/fx-scrape-and-impacts (cut from origin/master 17a784980).
Task: port rows 7, 8, 11 and 13 of the audit-effect-calls "Unported calls" table — rigid body
scrape dust (RB_Update 0x1d6e38), anim script commands 6 and 7 (AnimProcessScriptCmds
0x1c1b38), the laser env puff (Effect_Laser 0x1cef38:117) and the wall-puff velocity + the
HITDATA+0x51 roll (Effect_Bullet 0x1ce8a8). Merge origin/master first. Cite [M] for each call,
pin each with a test through the real path on real level data, report every value used. Shell
casings out of scope; do not change them.
