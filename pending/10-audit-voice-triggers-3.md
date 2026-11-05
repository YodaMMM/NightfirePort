# audit-voice-triggers-3 — guard voice triggers without shifting the walks

s/audit-voice-triggers wires 4 missing voice triggers but its extra random draws broke walkreal 01, 03 and smg_close_range. Rebase it on master. For each trigger check the exact draw order at the game's call site [M]. If the draws are right, fix the walks by re-deriving their routes; if a draw is extra, remove it. Run the walkreal binaries for 01 and 03 and smg_close_range and report the results.
