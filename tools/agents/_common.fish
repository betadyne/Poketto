#!/usr/bin/env fish

set -gx GITHUB_REPO "betadyne/Poketto"

set -l config_dir "$HOME/.config/poketto"

function load_github_token
    if set -q GITHUB_TOKEN; and test -n "$GITHUB_TOKEN"
        return 0
    end

    if test -f "$config_dir/github.env"
        source "$config_dir/github.env"
        if set -q GITHUB_TOKEN; and test -n "$GITHUB_TOKEN"
            set -gx GITHUB_TOKEN $GITHUB_TOKEN
            return 0
        end
    end

    set -l script_dir (dirname (status --current-filename))
    if test -f "$script_dir/.env"
        source "$script_dir/.env"
        if set -q GITHUB_TOKEN; and test -n "$GITHUB_TOKEN"
            set -gx GITHUB_TOKEN $GITHUB_TOKEN
            return 0
        end
    end

    echo "Error: GITHUB_TOKEN not found"
    echo ""
    echo "Set up your token using one of these methods:"
    echo "  1. Export: set -gx GITHUB_TOKEN 'your-token'"
    echo "  2. File: echo 'set -gx GITHUB_TOKEN your-token' > $config_dir/github.env"
    echo "  3. gh CLI: gh auth login"
    echo ""
    echo "Get a token at: https://github.com/settings/tokens"
    return 1
end

function check_gh_cli
    if not command -q gh
        echo "Error: GitHub CLI (gh) not found"
        echo ""
        echo "Install on Arch Linux:"
        echo "  sudo pacman -S github-cli"
        echo ""
        echo "Then authenticate:"
        echo "  gh auth login"
        return 1
    end
    return 0
end

function check_gh_auth
    if not gh auth status &>/dev/null
        echo "Error: Not authenticated with GitHub CLI"
        echo ""
        echo "Run: gh auth login"
        return 1
    end
    return 0
end

function init_agent_tools
    check_gh_cli; or exit 1
    
    load_github_token 2>/dev/null
    or check_gh_auth
    or exit 1
end

function print_issue
    set -l number $argv[1]
    set -l title $argv[2]
    set -l state $argv[3]
    set -l labels $argv[4]
    
    set -l state_icon "[ ]"
    if test "$state" = "closed"
        set state_icon "[x]"
    end
    
    echo "$state_icon #$number: $title"
    if test -n "$labels"
        echo "   Labels: $labels"
    end
end

init_agent_tools
