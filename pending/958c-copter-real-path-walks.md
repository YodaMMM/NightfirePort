## Your task: `copter-real-path-walks`, the helicopter missions must still finish with the faithful copter machine

`s/copter-hitdata-damage` (958b, built on 958 copter-path-flight) was taken back out of batch 1022ac by the main agent's suite. The faithful `Copter_Update` machine writes the out channel only on the death (state 4/6 once hp < damage [M 0x17dbb8:355-370]). Four real-path walks still expect the old stand-in flight to write it:

- `walkreal_07000008_live::walk_the_gondola_ride_to_the_end` (:353) "the helicopter's flight wrote ch 0x4B"
- `campaign_flow_live::campaign_flow_story_missions_1_to_3` and `::front_end_chain_01_to_05` (:370) "the gondola ride chain completes ... the copter → ch 0x4B"
- `walkreal_0700000b_live::the_roof_cutscene_060004fa_writes_ch59_ch57_and_arms_the_helicopter_gate` (:278) "the copter's flight → ch 0x66"
- `walkreal_0700000b_live::walk_balcony_to_the_roof_jump` (:147) "the helicopter flew (gate 0x2A, flight 060008BE 299 ticks) → out ch 0x66"

Start from `origin/s/copter-hitdata-damage`, merge `origin/master`, branch `s/copter-real-path-walks`.

1. For 07000008: drive the walks the way 958b's fixed `fix_objects2_live` gondola test does: Bond shoots the Hind down with real input until ch 0x4B. No damage pokes.
2. For 0700000b: find from the decompile and the disc how the game reaches ch 0x66 on the roof [M Copter_Update / the 0b scripts / D placements]. Does the player shoot that copter down too? Is 0x66 its out channel, its arrival (+0x72), or written by a script? If the walk's old expectation (the flight alone writes 0x66) is wrong, fix the test and cite the evidence. If the game really writes it without a kill, port that path. Note the death on 0x700000b goes straight to state 6 [M :361].
3. Re-apply two compile fixes the main agent made in batch 1022ac: `crates/nf-game/tests/difficulty_gates.rs` → `Copter::create(&placements, &[], difficulty)`; `crates/nf-view/tests/interact_07000005_live.rs` `.started` → `.state != 0`.
4. **Run the full suite to completion** on the final tree merged with origin/master and report the total line.
