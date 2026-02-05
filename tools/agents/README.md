# Agent Tooling Scripts

Fish shell scripts for AI coding agents and contributors working on Poketto.

## Requirements

- **Fish Shell**: `sudo pacman -S fish`
- **GitHub CLI**: `sudo pacman -S github-cli`
- **jq**: `sudo pacman -S jq`

## Setup

1. Authenticate with GitHub CLI:
   ```fish
   gh auth login
   ```

2. (Optional) Set up a token file for non-interactive use:
   ```fish
   mkdir -p ~/.config/poketto
   echo 'set -gx GITHUB_TOKEN your-token-here' > ~/.config/poketto/github.env
   ```

## Available Scripts

### Session Management

| Script | Description |
|--------|-------------|
| `session-start.fish` | Pick a ready P0 task and mark it in-progress |
| `session-update.fish` | Update status/comment on an issue |
| `pick-task.fish` | Show the top ready P0 issue |

### Git Operations

| Script | Description |
|--------|-------------|
| `git-sync.fish` | Pull latest changes from remote |
| `git-push.fish` | Push current branch to remote |
| `git-commit.fish` | Commit staged changes with a message |

### Issue Management

| Script | Description |
|--------|-------------|
| `issue-create.fish` | Create a new GitHub issue |
| `issue-list.fish` | List GitHub issues |
| `issue-update.fish` | Update an issue state/labels |

### Project Management

| Script | Description |
|--------|-------------|
| `project-list.fish` | List GitHub projects |
| `project-status.fish` | Show project board status |

## Usage Examples

### Start a coding session
```fish
fish tools/agents/session-start.fish --assign-self
fish tools/agents/session-update.fish --issue 42 --comment "Completed the backend implementation"
fish tools/agents/session-update.fish --issue 42 --status done
```

### Git workflow
```fish
fish tools/agents/git-sync.fish
fish tools/agents/git-commit.fish --type feat "add game sorting feature"
fish tools/agents/git-push.fish
```

### Issue management
```fish
fish tools/agents/issue-list.fish --labels "P0"
fish tools/agents/issue-create.fish --title "Fix crash on startup" --body "Description..." --labels "bug,P0"
fish tools/agents/issue-update.fish --issue 42 --state closed
```

## Label Conventions

- **Priority**: `P0` (critical), `P1` (high), `P2` (medium)
- **Status**: `ready`, `in-progress`, `blocked`
- **Type**: `bug`, `enhancement`, `docs`, `chore`
