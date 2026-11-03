# level-0c-yard-climb-real-health-3 — the 0c yard climb on real health

Branch from origin/s/level-0c-yard-climb-real-health-2 (merge origin/master into it first) and read its report (the blocked task file's report section). The real-health sister stays ignored behind two measured walls:

1. Sniper #18 cannot be hit from the yard floor: its twin #36 (alive, then its corpse) takes every round first [M Drone_CollisionHandler 0x140968]. Find a stand whose angle clears the twin, or deal with the twin's body the way a player would (kill order, a different approach).
2. The health budget is negative: Bond reaches the yard at ~33.6 hp after the en-route fights (#30/#37 300-tick stands), then #18/#15/#1 finish him. Cut the en-route damage (shorter stands against moving guards, better positions) and suppress #15/#1 before the crossing.

Real input only; no health top-ups or forced state; shared helpers opt-in. Done = the sister un-ignored and green, every walkreal_*_live green, then the full suite total line.
