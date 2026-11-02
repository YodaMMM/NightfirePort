# level-15-mines-shot-from-range — shoot the laser mines from a distance on real input

Branch from origin/master.

On 07000015 the floating laser mines (the nine 0xFE placements, Mine_Update 0x17c930) should explode when shot from range; merges level-15-mines and mines-tripwires changed this, but no test reaches a mine through real input from a distance.

Do:
1. Walk a real route (no teleport after load, no ammo/health writes) to a spot with a clear line to a mine at range (>8 units), aim with the level's own weapons (what the level gives Bond [D loadout/pickups]), and fire.
2. The mine must take the hit through the game's bullet path [M the bullet-vs-object pick] and explode (its beam goes, the blast plays, the mine is gone). If it does not, find why and fix it, citing the decompile.
3. Add the live test; run interactables_07000015_live, walkreal_07000015_live, mine_bob_live, then the full suite.
