#!/usr/bin/env fish

set -l script_dir (dirname (status --current-filename))
source "$script_dir/_common.fish"

echo "Fetching GitHub Projects..."
echo ""

set -l owner (string split "/" $GITHUB_REPO)[1]

set -l projects (gh project list --owner $owner --format json 2>/dev/null)

if test -z "$projects" -o "$projects" = "[]"
    echo "No projects found for $owner"
    echo ""
    echo "Create a project at: https://github.com/orgs/$owner/projects/new"
    echo "Or for personal: https://github.com/users/$owner/projects/new"
    exit 0
end

echo "Projects for $owner:"
echo "===================="
echo ""

echo $projects | jq -r '.projects[] | "#\(.number): \(.title)\n   URL: \(.url)\n   Items: \(.items.totalCount)\n"'

echo ""
echo "View project details: fish tools/agents/project-status.fish"
