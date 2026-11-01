# WORKLOG

## per-guard-motion-stamps (started 2026-11-01 18:48 UTC)

Task: the port keeps ONE global first-moved / first-stopped stamp for the player's motion; the
game keeps one per guard (Drone+0x224, the +0x1DD/+0x1DF latches [M DroneWeap_DoOpponentTargetting
0x179a48:47-73]). Port the per-guard stamps (each guard latches when IT observes the change), with
the addresses, and pin: two guards that start tracking at different ticks clear their first-moved
terms at different ticks (the global model would clear them together). Branch cut from
origin/master 6042ee53. Plan: read DroneWeap_DoOpponentTargetting 0x179a48 decompile + find the
port's global stamp, port per-guard latches, reproduce first (two-guards test through the real
input path), run enemy_accuracy_live, ai_live, every walkreal_* real-damage variant, nf-game, full
suite, report.
