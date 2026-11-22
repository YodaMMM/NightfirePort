## real-health-00agent-01 (2026-11-22, s/real-health-00agent-01)

Task: the 01 00 Agent real-damage contact walk
`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`
is ignored on the s/hard-walks-d ledger: Bond leaves the ch-0xF pocket at 57.4 (the hit-box
shield + torso aim lane stand) but the stand kills only #17; #12 and the captain #3 outlast
it behind the contact, and the contact dies in the pocket at t50960 (mission fail). The
driver's order: FIRST check whether the contact's death is the port doing something the game
does not (guard target choice against the contact, his health, cover), then re-route. Branch
base master dbf17a37d + origin/s/hard-walks-d merged (its ledger is the newest on this test
and is not on master yet). Helpers stay d3-gated; real input only, no top-ups, never lower
damage or health, never change guard code. Un-ignore when green.
