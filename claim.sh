#!/usr/bin/env bash
# Claim the next pending task from the queue branch.
# Usage (from your NightfirePort clone): bash <queue dir>/claim.sh <worker-name>
# Prints "TASK <id> <path to the task file>" or "EMPTY". The claim is atomic: if another worker
# pushed first, it retries with the next task.
set -u
W="${1:?worker name}"
Q="$(cd "$(dirname "$0")" && pwd)"
cd "$Q"
for attempt in 1 2 3 4 5 6 7 8; do
  git fetch -q origin queue && git reset -q --hard origin/queue || { sleep 5; continue; }
  # A pending task that's also in claimed/, done/ or blocked/ is a leftover copy (a stale push re-added it on
  # 2026-10-21 and 923 was claimed five times): drop it instead of claiming it, then pick the next one.
  f=""
  for c in $(ls pending/*.md 2>/dev/null | sort); do
    cid="$(basename "$c" .md)"
    if [ -e "claimed/$cid.md" ] || [ -e "done/$cid.md" ] || [ -e "blocked/$cid.md" ]; then
      git rm -q "$c"; git commit -qm "drop leftover pending copy of $cid (already claimed/done/blocked)"; continue
    fi
    f="$c"; break
  done
  if [ -z "$f" ]; then git push -q origin HEAD:queue 2>/dev/null; echo "EMPTY"; exit 0; fi
  id="$(basename "$f" .md)"
  git mv "$f" "claimed/$id.md" || { sleep 2; continue; }
  printf '\n---\nclaimed-by: %s at %s\n' "$W" "$(date -u +%Y-%m-%dT%H:%MZ)" >> "claimed/$id.md"
  git add "claimed/$id.md"
  git commit -qm "claim $id ($W)"
  if git push -q origin HEAD:queue 2>/dev/null; then
    echo "TASK $id $Q/claimed/$id.md"; exit 0
  fi
  sleep $((RANDOM % 5 + 1))
done
echo "CLAIM-FAILED (could not push after 8 attempts)"; exit 1
