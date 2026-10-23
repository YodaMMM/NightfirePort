# WORKLOG

## enemy-mission-kind8
Started 2026-10-23 16:55 UTC. Branch s/enemy-mission-kind8 from origin/master a67f1a27e.
Task: NDrone2_DSTATE_EnemyMission 0x15e1e8 and its kind-8 paths (AssignAIPath(8)) are not
ported (drone.rs notes it). Find which levels place guards that use it [D placements: the
dtype / mission path params], port the state and the path assignment from the decompile [M],
and add a real-path test on a level that uses it: the guard follows its mission path.
Sister: a guard without a kind-8 path never enters the state.
Plan:
1. Read the decompile of NDrone2_DSTATE_EnemyMission 0x15e1e8 and the kind-8 (mission) path
   assignment; check the disassembly wherever the C drops stores/args.
2. Find the disc placements (dtype / mission path params) of guards that use kind-8.
3. Port the state + path assignment into nf-game; real-path test on a level that uses it
   (guard follows its mission path); sister test (no kind-8 path => never enters the state).
4. Full suite via tools/test/suite.py before pushes.

Work log:
- (16:55) branch created, WORKLOG started; repo/ISO/work/queue-worktree already present from
  the earlier session; token refreshed and origin fetched.
