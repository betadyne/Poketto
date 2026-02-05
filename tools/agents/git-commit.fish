#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
set -l repo_root (dirname (dirname $script_dir))

cd $repo_root

set -l commit_type ""
set -l commit_message ""
set -l add_all false

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --type -t
            set i (math $i + 1)
            set commit_type $argv[$i]
        case --all -a
            set add_all true
        case '*'
            if test -z "$commit_message"
                set commit_message $argv[$i]
            else
                set commit_message "$commit_message $argv[$i]"
            end
    end
    set i (math $i + 1)
end

if test -z "$commit_message"
    echo "Error: Commit message is required"
    echo ""
    echo "Usage: fish tools/agents/git-commit.fish \"message\""
    echo "       fish tools/agents/git-commit.fish --type feat \"message\""
    echo "       fish tools/agents/git-commit.fish --all \"message\""
    echo ""
    echo "Types: feat, fix, docs, refactor, chore, test"
    exit 1
end

if test "$add_all" = true
    echo "Staging all changes..."
    git add -A
end

set -l staged (git diff --cached --name-only)
if test -z "$staged"
    echo "Warning: No staged changes to commit"
    echo ""
    echo "Stage changes first:"
    echo "  git add <file>        # Stage specific file"
    echo "  git add -A            # Stage all changes"
    echo "  fish tools/agents/git-commit.fish --all \"message\""
    exit 1
end

set -l full_message $commit_message
if test -n "$commit_type"
    set full_message "$commit_type: $commit_message"
end

echo "Committing with message: $full_message"
echo ""
echo "Staged files:"
for file in $staged
    echo "  - $file"
end
echo ""

git commit -m "$full_message"

if test $status -eq 0
    echo ""
    echo "Commit created successfully!"
    echo ""
    git log --oneline -1
    echo ""
    echo "Push with: fish tools/agents/git-push.fish"
else
    echo ""
    echo "Commit failed"
    exit 1
end
