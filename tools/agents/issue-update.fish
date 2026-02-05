#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l issue_num ""
set -l state ""
set -l add_labels ""
set -l remove_labels ""

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --issue -i
            set i (math $i + 1)
            set issue_num $argv[$i]
        case --state -s
            set i (math $i + 1)
            set state $argv[$i]
        case --add-label
            set i (math $i + 1)
            set add_labels $argv[$i]
        case --remove-label
            set i (math $i + 1)
            set remove_labels $argv[$i]
    end
    set i (math $i + 1)
end

if test -z "$issue_num"
    echo "Error: --issue is required"
    echo ""
    echo "Usage: fish tools/agents/issue-update.fish --issue <#> --state <open|closed>"
    echo "       fish tools/agents/issue-update.fish --issue <#> --add-label \"P0\""
    echo "       fish tools/agents/issue-update.fish --issue <#> --remove-label \"P1\""
    exit 1
end

echo "Updating issue #$issue_num..."

if test -n "$state"
    switch $state
        case open
            echo "Reopening issue..."
            gh issue reopen $issue_num --repo "$GITHUB_REPO" 2>/dev/null
        case closed
            echo "Closing issue..."
            gh issue close $issue_num --repo "$GITHUB_REPO" 2>/dev/null
        case '*'
            echo "Error: Invalid state: $state (use: open, closed)"
            exit 1
    end
end

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

echo ""
echo "Issue #$issue_num updated"

echo ""
gh issue view $issue_num --repo "$GITHUB_REPO" --json number,title,state,labels | jq -r '"#\(.number): \(.title)\nState: \(.state)\nLabels: \(.labels | map(.name) | join(", "))"'
