#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
repo="${1:-IcarusExE/yks-pusulasi}"
# Verify access before changing the remote or uploading source.
gh repo view "$repo" --json nameWithOwner --jq .nameWithOwner
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo 'Commit local changes before publishing.' >&2
  exit 1
fi
gh auth setup-git
git remote set-url origin "https://github.com/${repo}.git"
git push -u origin main
if ! gh api "repos/${repo}/pages" --silent 2>/dev/null; then
  gh api --method POST "repos/${repo}/pages" -f build_type=workflow --silent
else
  gh api --method PUT "repos/${repo}/pages" -f build_type=workflow --silent
fi
printf 'Source pushed. Deployment: https://github.com/%s/actions\n' "$repo"
