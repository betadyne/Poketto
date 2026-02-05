#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l title ""
set -l body ""
set -l labels ""

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --title -t
            set i (math $i + 1)
            set title $argv[$i]
        case --body -b
            set i (math $i + 1)
            set body $argv[$i]
        case --labels -l
            set i (math $i + 1)
            set labels $argv[$i]
    end
    set i (math $i + 1)
end

if test -z "$title"
    echo "Error: --title is required"
    echo ""
    echo "Usage: fish tools/agents/issue-create.fish --title \"Title\" --body \"Body\" [--labels \"bug,P0\"]"
    echo ""
    echo "Examples:"
    echo "  fish tools/agents/issue-create.fish --title \"Fix login bug\" --body \"Description...\" --labels \"bug,P0\""
    echo "  fish tools/agents/issue-create.fish -t \"New feature\" -b \"Details...\" -l \"enhancement,P1\""
    exit 1
end

echo "Creating issue..."
echo "   Title: $title"

set -l cmd "gh issue create --repo $GITHUB_REPO --title \"$title\""

if test -n "$body"
    set cmd "$cmd --body \"$body\""
end

if test -n "$labels"
    set cmd "$cmd --label \"$labels\""
end

set -l result (eval $cmd 2>&1)

if test $status -eq 0
    echo ""
    echo "Issue created successfully!"
    echo "URL: $result"
else
    echo ""
    echo "Failed to create issue:"
    echo $result
    exit 1
end
