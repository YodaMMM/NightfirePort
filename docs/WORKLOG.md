# WORKLOG

## level-16-stuck-overlook (branch s/level-16-stuck-overlook)

- start (master 521d9bb2). Task: on 07000016 at Bond (-17.86, 6.81, -97.16) facing yaw 6.98
  (the walkway beside the big control-room window, around tick 3700 of a real run) the
  player cannot continue the level, and the held rifle's view model visibly clips into the
  glass/wall. Find what the game expects here [D the route on from this walkway: doors,
  triggers, the channel that opens the way; M the classes], reproduce on real input from
  the level start with the port's walker, fix any port gap with the address cited, and pin
  a walk that passes this spot. Also check the view model's near-clip / wall push-back at
  that spot against the game [M the view-model draw path]. Run walkreal_07000016_live,
  interactables_07000016_live and nf-game; quote the totals.
