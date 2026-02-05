#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

set -l project_num ""

set -l i 1
while test $i -le (count $argv)
    switch $argv[$i]
        case --project -p
            set i (math $i + 1)
            set project_num $argv[$i]
    end
    set i (math $i + 1)
end

set -l owner (string split "/" $GITHUB_REPO)[1]

echo "Fetching project status..."
echo ""

if test -z "$project_num"
    set -l projects (gh project list --owner $owner --format json --limit 1 2>/dev/null)
    if test -z "$projects" -o "$projects" = "null"
        echo "No projects found. Create one at GitHub."
        exit 0
    end
    set project_num (echo $projects | jq -r '.projects[0].number')
end

echo "Project #$project_num for $owner"
echo "================================"
echo ""

set -l items (gh project item-list $project_num --owner $owner --format json --limit 50 2>/dev/null)

if test -z "$items" -o "$items" = "null"
    echo "No items in project or project not found."
    exit 0
end

set -l todo_items
set -l in_progress_items
set -l done_items
set -l other_items

for item in (echo $items | jq -c '.items[]')
    set -l title (echo $item | jq -r '.title')
    set -l item_type (echo $item | jq -r '.type')
    set -l status (echo $item | jq -r '.status // "No Status"')
    
    set -l line "- $title ($item_type)"
    
    switch $status
        case "Todo" "To Do" "Backlog" "Ready"
            set todo_items $todo_items "$line"
        case "In Progress" "In progress" "Doing"
            set in_progress_items $in_progress_items "$line"
        case "Done" "Completed" "Closed"
            set done_items $done_items "$line"
        case '*'
            set other_items $other_items "$line [$status]"
    end
end

if test (count $todo_items) -gt 0
    echo "To Do: ("(count $todo_items)")"
    for item in $todo_items
        echo "   $item"
    end
    echo ""
end

if test (count $in_progress_items) -gt 0
    echo "In Progress: ("(count $in_progress_items)")"
    for item in $in_progress_items
        echo "   $item"
    end
    echo ""
end

if test (count $done_items) -gt 0
    echo "Done: ("(count $done_items)")"
    for item in $done_items
        echo "   $item"
    end
    echo ""
end

if test (count $other_items) -gt 0
    echo "Other:"
    for item in $other_items
        echo "   $item"
    end
    echo ""
end

echo "---"
set -l total (math (count $todo_items) + (count $in_progress_items) + (count $done_items) + (count $other_items))
echo "Total items: $total"
