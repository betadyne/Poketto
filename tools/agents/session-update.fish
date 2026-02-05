#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l issue_num ""
set -l status ""
set -l comment ""

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --issue -i
            set i (math $i + 1)
            set issue_num $argv[$i]
        case --status -s
            set i (math $i + 1)
            set status $argv[$i]
        case --comment -c
            set i (math $i + 1)
            set comment $argv[$i]
    end
    set i (math $i + 1)
end

if test -z "$issue_num"
    echo "Error: --issue is required"
    echo ""
    echo "Usage: fish tools/agents/session-update.fish --issue <#> --status <status> [--comment \"message\"]"
    echo ""
    echo "Status options: ready, in-progress, blocked, done"
    exit 1
end

set -l add_labels ""
set -l remove_labels ""
set -l close_issue false

switch $status
    case ready
        set add_labels "ready"
        set remove_labels "in-progress,blocked"
    case in-progress
        set add_labels "in-progress"
        set remove_labels "ready,blocked"
    case blocked
        set add_labels "blocked"
        set remove_labels "ready,in-progress"
    case done
        set remove_labels "ready,in-progress,blocked"
        set close_issue true
    case ""
    case '*'
        echo "Error: Invalid status: $status"
        echo "   Valid options: ready, in-progress, blocked, done"
        exit 1
end

echo "Updating issue #$issue_num..."

if test -n "$add_labels"
    echo "Adding labels: $add_labels"
    gh issue edit $issue_num --repo "$GITHUB_REPO" --add-label "$add_labels" 2>/dev/null
end

if test -n "$remove_labels"
    echo "Removing labels: $remove_labels"
    for label in (string split "," $remove_labels)
        gh issue edit $issue_num --repo "$GITHUB_REPO" --remove-label "$label" 2>/dev/null
    end
end

if test -n "$comment"
    echo "Posting comment..."
    set -l timestamp (date -Iseconds)
    set -l status_text ""
    if test -n "$status"
        set status_text "**Status:** $status"
    end
    
    gh issue comment $issue_num --repo "$GITHUB_REPO" --body "## Update

$status_text

$comment

---
*Posted via session-update.fish at $timestamp*" 2>/dev/null
end

if test "$close_issue" = true
    echo "Closing issue..."
    gh issue close $issue_num --repo "$GITHUB_REPO" 2>/dev/null
    
    set -l timestamp (date -Iseconds)
    gh issue comment $issue_num --repo "$GITHUB_REPO" --body "## Session Complete

**Timestamp:** $timestamp

Issue resolved and closed.

---
*Posted via session-update.fish*" 2>/dev/null
end

echo ""
echo "Issue #$issue_num updated successfully"
