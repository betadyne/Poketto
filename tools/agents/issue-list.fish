#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l state "open"
set -l labels ""
set -l limit 20

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --state -s
            set i (math $i + 1)
            set state $argv[$i]
        case --labels -l
            set i (math $i + 1)
            set labels $argv[$i]
        case --limit -n
            set i (math $i + 1)
            set limit $argv[$i]
    end
    set i (math $i + 1)
end

echo "Fetching issues..."
echo ""

set -l cmd "gh issue list --repo $GITHUB_REPO --state $state --limit $limit --json number,title,state,labels,assignees,createdAt"

if test -n "$labels"
    set cmd "$cmd --label \"$labels\""
end

set -l issues (eval $cmd 2>/dev/null)

if test -z "$issues" -o "$issues" = "[]"
    echo "No issues found matching criteria."
    exit 0
end

set -l p0_issues
set -l p1_issues
set -l p2_issues
set -l other_issues

for issue in (echo $issues | jq -c '.[]')
    set -l num (echo $issue | jq -r '.number')
    set -l title (echo $issue | jq -r '.title')
    set -l issue_state (echo $issue | jq -r '.state')
    set -l issue_labels (echo $issue | jq -r '.labels | map(.name) | join(", ")')
    set -l assignees (echo $issue | jq -r '.assignees | map(.login) | join(", ")')
    
    set -l line "#$num: $title"
    if test -n "$assignees"
        set line "$line [$assignees]"
    end
    
    if echo $issue_labels | string match -q "*P0*"
        set p0_issues $p0_issues "$line"
    else if echo $issue_labels | string match -q "*P1*"
        set p1_issues $p1_issues "$line"
    else if echo $issue_labels | string match -q "*P2*"
        set p2_issues $p2_issues "$line"
    else
        set other_issues $other_issues "$line"
    end
end

if test (count $p0_issues) -gt 0
    echo "P0 - Critical:"
    for issue in $p0_issues
        echo "   $issue"
    end
    echo ""
end

if test (count $p1_issues) -gt 0
    echo "P1 - High:"
    for issue in $p1_issues
        echo "   $issue"
    end
    echo ""
end

if test (count $p2_issues) -gt 0
    echo "P2 - Medium:"
    for issue in $p2_issues
        echo "   $issue"
    end
    echo ""
end

if test (count $other_issues) -gt 0
    echo "Other:"
    for issue in $other_issues
        echo "   $issue"
    end
    echo ""
end

echo "---"
echo "Total: "(echo $issues | jq '. | length')" issues"
