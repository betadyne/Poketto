#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l assign_self false
for arg in $argv
    switch $arg
        case --assign-self -a
            set assign_self true
    end
end

echo "Looking for ready P0 issues..."

set -l issues (gh issue list --repo "$GITHUB_REPO" --label "P0" --state open --json number,title,labels,assignees --limit 10 2>/dev/null)

if test -z "$issues" -o "$issues" = "[]"
    echo "No open P0 issues found"
    exit 1
end

set -l selected_issue ""
set -l selected_title ""

for issue in (echo $issues | jq -c '.[]')
    set -l num (echo $issue | jq -r '.number')
    set -l title (echo $issue | jq -r '.title')
    set -l assignees (echo $issue | jq -r '.assignees | length')
    
    set -l in_progress (echo $issue | jq -r '.labels[] | select(.name == "in-progress") | .name')
    
    if test -z "$in_progress" -a "$assignees" -eq 0
        set selected_issue $num
        set selected_title $title
        break
    end
end

if test -z "$selected_issue"
    echo "Warning: All P0 issues are either assigned or in-progress"
    echo ""
    echo "Available P0 issues:"
    echo $issues | jq -r '.[] | "  #\(.number): \(.title)"'
    exit 1
end

echo "Selected issue #$selected_issue: $selected_title"

echo "Adding 'in-progress' label..."
gh issue edit $selected_issue --repo "$GITHUB_REPO" --add-label "in-progress" 2>/dev/null

if test "$assign_self" = true
    echo "Assigning to self..."
    gh issue edit $selected_issue --repo "$GITHUB_REPO" --add-assignee "@me" 2>/dev/null
end

echo "Posting session start comment..."
set -l timestamp (date -Iseconds)
gh issue comment $selected_issue --repo "$GITHUB_REPO" --body "## Session Start

**Timestamp:** $timestamp

Starting work on this issue.

---
*Posted via session-start.fish*" 2>/dev/null

echo ""
echo "Session started for issue #$selected_issue"
echo "   Title: $selected_title"
echo ""
echo "Next steps:"
echo "   1. Work on the issue"
echo "   2. Post updates: fish tools/agents/session-update.fish --issue $selected_issue --comment \"progress\""
echo "   3. End session: fish tools/agents/session-update.fish --issue $selected_issue --status done"
