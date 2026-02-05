#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
set -l repo_root (dirname (dirname $script_dir))

cd $repo_root

set -l use_rebase false
for arg in $argv
    switch $arg
        case --rebase -r
            set use_rebase true
    end
end

echo "Syncing with remote..."
echo ""

set -l status_output (git status --porcelain)
if test -n "$status_output"
    echo "Warning: You have uncommitted changes:"
    git status --short
    echo ""
    echo "Stashing changes before pull..."
    git stash push -m "Auto-stash before sync"
    set -l stashed true
else
    set -l stashed false
end

echo "Fetching from origin..."
git fetch origin

set -l current_branch (git branch --show-current)
echo "Current branch: $current_branch"

set -l upstream (git rev-parse --abbrev-ref @{u} 2>/dev/null)
if test -z "$upstream"
    echo "Warning: No upstream branch set for $current_branch"
    echo "   Run: git push -u origin $current_branch"
    exit 0
end

set -l behind (git rev-list --count HEAD..$upstream)
set -l ahead (git rev-list --count $upstream..HEAD)

echo "Status: $ahead ahead, $behind behind $upstream"

if test $behind -eq 0
    echo "Already up to date!"
else
    if test "$use_rebase" = true
        echo "Rebasing on $upstream..."
        git rebase $upstream
    else
        echo "Pulling from $upstream..."
        git pull --ff-only origin $current_branch
        or begin
            echo "Warning: Fast-forward not possible. Try:"
            echo "   fish tools/agents/git-sync.fish --rebase"
            exit 1
        end
    end
end

if set -q stashed; and test "$stashed" = true
    echo ""
    echo "Restoring stashed changes..."
    git stash pop
end

echo ""
echo "Sync complete!"
git log --oneline -3
