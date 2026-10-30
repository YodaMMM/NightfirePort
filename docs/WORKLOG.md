## level-0d-crane-transit (started 03:03 UTC 2026-10-30)

Task: 0700000d the crane route into the ST-3 yard.
The walk test reaches the yard through a documented position stand-in; the game's way in
is the crane sequence (hint [D USATxt 0x4000064]; cranes are 0xE2 SimpleScripts -
shipping_handle #112, script players #115/#492, buttons #113/#114 animating on ch 0x2b)
which the port does not simulate [docs/levels/0700000d.md §6]. Find what the game does:
which input starts each crane, how the 0xE2 SimpleScript moves its load, how the player
gets carried [M the 0xE2 create/update functions, D the scripts]. Port it so the player
can do the crane route on real input, replace the stand-in in the walk with that route
(remove the set_channel of 0x1b/0x1c at line ~38 if the crane path raises them). Say in
the report whether the same 0xE2 family covers 0c's vent/pulley stand-ins. FULL suite,
quote its total line.

Plan: branch pushed from origin/master (21fbeed3) now; downloads running next; then read
the 0xE2 SimpleScript decomp + the 0700000d scripts, port, re-walk the test on real
input.
