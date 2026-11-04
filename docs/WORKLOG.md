
## glass-pass-every-level-3 (2026-11-04)

Branch s/glass-pass-every-level-3 (from origin/s/glass-pass-every-level-2, merged origin
master 182408f88). The 01 impact half: CLOSED — the NPC freeze releases wake+2 (npc_enabled
false t0 → true t2121 [L the probe; M the Drone_EnableAll/NPCGlobals gate drone.rs:14678]),
the taps aim the real hit boxes and only ENABLED guards ([M Collide_PickObj 0x1eae80:26-30 —
the disabled drop]); the run logs the first impact t2126, the pane 130 drain t2131, and the
closing shot t2448 (pane 0 drains + impact (0,4) same window). The 08 route half: CLOSED —
the doorway's clear span is x 106.6..107.6 with the leaf OPEN (the old 107.8 aim was the wall
face the capsule hung on, grounded=false, 6000 ticks [L the passage scan]); the in_booth
assert holds. The 08 glass half: sheet 177 BROKE on the real-input tap (post-phase hp −3.50
dead true); the open half is the round's reach — the tapped guard repositions mid-flight (he
ran, no hp drop) — #[ignore]d with the measured state in the reason. The helper now requires
the REAL pane span (the ±0.6 slack aimed rounds through fresh air beside the glass) and
pulses the semi-auto trigger. No crates/*/src changes. The glass binary whole, final tree:
3/0/3. NOT run this window: the walkreal sweep builds and the full suite (the 100-minute
window went to the 01/08 measurements above) — the next session runs both on this tree.
