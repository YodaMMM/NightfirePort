# 0700000c: the hip pull on the deck sentry

`upward_shot_0700000c_live::the_hip_pull_dries_three_clips_on_the_deck_sentry` fails on the current tree. Find whether the sentry dies too early, rounds miss, or the clip/reload counts differ from the game (weapon record clip size, reload, the upward-aim spread), compare with the decompile, and fix the game code or the test's route if the old expectation was wrong, with evidence.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0c-deck-sentry-clips`, report in docs/reports/level-0c-deck-sentry-clips.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-08T17:00Z
