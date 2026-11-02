# real-health-walks-08-11-12-14-15 — finish each level on real health

Branch from origin/master.

These levels have whole-mission walks (walkreal_<level>_live) that top Bond's health up every tick: 07000008 07000011 07000012 07000014 07000015. None has a real-health variant pinned (01/02/04-07/09/0a/13/16/1b/41/4a do — see walk_..._real_damage there: the walker's top_up off, die_panics on).

Do, per level:
1. Add a real-health variant of the level's whole-mission walk (no health/armour/ammo writes, no teleports after load, auto-aim as the app default). The walk must reach the level's end.
2. Where Bond dies, read the damage log (who, weapon, range) and decide: if the port's guards deal more than the game [M the drone weapon/accuracy code], fix the port and cite it; otherwise give the walker a tactic the level really offers (cover, pickups on the route, kill order).
3. Run each walkreal_<level>_live and every real-damage walk elsewhere (shared walker changes must not break them), then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T14:15Z

---
## Report (done, 2026-11-02T15:23Z)

