#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l show_all false
for arg in $argv
    switch $arg
        case --all -a
            set show_all true
    end
end

echo "Fetching P0 issues..."

set -l issues (gh issue list --repo "$GITHUB_REPO" --label "P0" --state open --json number,title,labels,assignees,createdAt --limit 20 2>/dev/null)

if test -z "$issues" -o "$issues" = "[]"
    echo "No open P0 issues - backlog is clear!"
    exit 0
end

echo ""
echo "Open P0 Issues:"
echo "=================="
echo ""

set -l found_ready false

for issue in (echo $issues | jq -c '.[]')
    set -l num (echo $issue | jq -r '.number')
    set -l title (echo $issue | jq -r '.title')
    set -l assignees (echo $issue | jq -r '.assignees | map(.login) | join(", ")')
    set -l labels (echo $issue | jq -r '.labels | map(.name) | join(", ")')
    set -l created (echo $issue | jq -r '.createdAt' | string sub -l 10)
    
    set -l status "ready"
    if echo $labels | string match -q "*in-progress*"
        set status "in-progress"
    else if echo $labels | string match -q "*blocked*"
        set status "blocked"
    end
    
    set -l icon "[R]"
    switch $status
        case in-progress
            set icon "[P]"
        case blocked
            set icon "[B]"
    end
    
    if test "$show_all" = true -o "$status" = "ready"
        echo "$icon #$num: $title"
        echo "   Status: $status | Created: $created"
        if test -n "$assignees"
            echo "   Assignee: $assignees"
        end
        echo ""
        
        if test "$status" = "ready" -a "$found_ready" = false
            set found_ready true
        end
    end
end

if test "$found_ready" = false -a "$show_all" = false
    echo "Warning: No ready P0 issues. All are in-progress or blocked."
    echo ""
    echo "Run with --all to see all P0 issues."
end

echo ""
echo "To start working: fish tools/agents/session-start.fish"
