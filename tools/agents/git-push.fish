#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
set -l repo_root (dirname (dirname $script_dir))

cd $repo_root

set -l force_push false
set -l set_upstream false

for arg in $argv
    switch $arg
        case --force -f
            set force_push true
        case --set-upstream -u
            set set_upstream true
    end
end

set -l current_branch (git branch --show-current)
echo "Current branch: $current_branch"

set -l status_output (git status --porcelain)
if test -n "$status_output"
    echo "Warning: You have uncommitted changes:"
    git status --short
    echo ""
    echo "Commit or stash changes first."
    exit 1
end

set -l upstream (git rev-parse --abbrev-ref @{u} 2>/dev/null)
if test -n "$upstream"
    set -l ahead (git rev-list --count $upstream..HEAD)
    if test $ahead -eq 0
        echo "Nothing to push - up to date with $upstream"
        exit 0
    end
    echo "$ahead commit(s) to push"
else
    echo "No upstream set - will push with -u"
    set set_upstream true
end

set -l push_cmd "git push"

if test "$set_upstream" = true
    set push_cmd "$push_cmd -u origin $current_branch"
else
    set push_cmd "$push_cmd origin $current_branch"
end

if test "$force_push" = true
    echo "Warning: Force pushing (--force-with-lease for safety)..."
    set push_cmd "$push_cmd --force-with-lease"
end

echo "Pushing to origin/$current_branch..."
eval $push_cmd

if test $status -eq 0
    echo ""
    echo "Push successful!"
    
    set -l remote_url (git remote get-url origin | string replace ".git" "" | string replace "git@github.com:" "https://github.com/")
    echo "View: $remote_url/tree/$current_branch"
else
    echo ""
    echo "Push failed"
    exit 1
end
