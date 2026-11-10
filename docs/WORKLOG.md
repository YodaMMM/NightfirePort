
## level-13-warm-chain-margin — branch cut

Branch cut from origin/master ef7822e8b. Task: Chain Reaction's (07000013) ignored walk
walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage dies on
master at tick 2448 at (4.03,-368.39,33.35) with 3 hp after a 3.50 hit from drone #5 (weapon
0x19); the same route passed on d09f8b2b3. Bisect the walk across d09f8b2b3..master for the bleed
change, judge the new damage against the decompile [M], then re-route or re-tune the walk on real
input only (no top-ups, never lower damage or health) so it passes with a real margin; un-ignore
it. Run the whole walkreal_07000013_live binary and interactables_07000013_live.
