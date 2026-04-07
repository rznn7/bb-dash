# bb-dash

A TUI client for Bitbucket — manage pull requests entirely from the terminal.

> :construction: Work in progress

## Features

- List pull requests for the current repo with approval status at a glance
- View PR details: reviewers, build statuses, description
- Approve, unapprove, request changes, or unrequest changes directly from the TUI
- Open a PR in the browser or copy its link to the clipboard
- In-app help popup with keybinding reference

## Setup

Requires two environment variables (or a `.env` file):

```sh
BITBUCKET_USERNAME=your-username
BITBUCKET_API_TOKEN=your-app-password
```

## Stack

Rust with Tokio and Ratatui
