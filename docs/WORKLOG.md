
## real-health-00agent-01-hall-5
Started 2026-11-14 02:26 UTC. Branch s/real-health-00agent-01-hall-5 cut from origin/master
9182c25b4, then origin/s/real-health-00agent-01-hall-4 (4be0f61f5) merged in; its
docs/WORKLOG.md and docs/reports/* dropped from the merge result (merge commit cb1d8dc2c).
Task: hall-4 was measured on the old base 2265b403e; on the merged tree the 00 Agent walk
dies EARLIER than master alone (t3546 at (7.40,5.99,8.39), #6 skin 0x5000036 0x1c at 6.6 u,
vs master alone t6054 at (7.40,6.47,10.55)). Re-measure every lever on the merged tree and
re-tune until the 00 Agent walk gets at least as far as master alone, ideally past the AND
gate. Every lever gated difficulty >= 3; the d1/d2 walks stay byte-identical.

Work log:
- (02:26-02:35) repo found intact, remote token refreshed, fetched master/queue; branch
  created from origin/master 9182c25b4; hall-4 merged (auto, clean), its docs dropped and
  the merge amended (cb1d8dc2c). Checkpoint pushed next.
- (02:37-03:15) the merged-stream door phase measured on 4 instrumented runs ([L]
  /tmp/h5_run0-4.txt, all real-input): run0/1 reproduced the t3546 death exactly (16 pellet
  hits from #6 in exactly two windows t3433-34/t3545-46, nothing else for 2900 ticks — the
  return-fire fight held Bond standing in the mouth ray); run2 (the hall-5 entry reorder:
  deep line FIRST with the door closed, then the crouched press, then the crouched home —
  new opt-in Walker::use_door_crouch) was clean to the tick the stand restored — #7 (the
  DEEP captain, 0x5000038 0x1c, now parked at (8.3,6.3,17.3)) landed one full 8-pellet
  volley (7x7.2+4.6 = the whole 55.0) at (6.9,8.4); run3 (the crouch kept through the lure +
  hold; the crouch fight re-enabled for d3) — the crouched QUIET phases took ZERO fire
  (entry + 300-tick crouched lure) but the fight window still died to #7's volley at the
  same stream tick t3585; run4 (NF_HEALTH_TRACE) — NO drone took damage through the whole
  door phase: exposed_aim is None from the south structure the entire time (the fight's
  aim-only trigger never fired). The run3 lane grid is the map: the merged stream's south
  wall has the SECOND opening — full-height crouched flight lanes from the west stands
  (4,7)/(4.6,8.5) to BOTH #6 and #7, the same lane their pellets use. Next lever: the
  lane-triggered crouch taps (the corridor-fight shape) from the west stands.
- (03:15) checkpoint: the entry reorder + the use_door_crouch opt-in + the d3 crouch-fight
  re-enable pushed; the lane-trigger fight is the next edit.
