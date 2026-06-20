# rust-git-fsmonitor

A [watchman](https://github.com/facebook/watchman/)-based `core.fsmonitor` hook for git, written in Rust.

Git's fsmonitor support speeds up commands like `status` and `diff` on large
repos by asking an external file-watching service which paths changed since
the last query, instead of stat-ing the whole tree. Git itself doesn't know
how to talk to watchman — it just execs a hook with a version and an opaque
token, and expects a NUL-separated list of changed paths back. This binary is
that hook: it speaks the fsmonitor hook protocol on one side and the watchman
wire protocol on the other.

## Prerequisites

You need a watchman-compatible daemon running (or resolvable via the
`watchman` CLI on `PATH`).

- **[watchwoman](https://github.com/radiosilence/watchwoman)** (recommended) —
  a drop-in, wire-compatible reimplementation of watchman in Rust. Lower
  memory use, faster cold scans/queries, and it ships a `watchman`-named
  alias binary, so this hook (and anything else that speaks the watchman
  protocol) keeps working with no configuration changes.
- **[watchman](https://github.com/facebook/watchman/)** — the original
  Facebook implementation also works.

Either way, this tool talks to whatever `watchman_client`'s `Connector`
discovers: the socket at `$WATCHMAN_SOCK` if set, otherwise whatever
`watchman get-sockname` (or the `watchwoman` alias of it) reports from
`PATH`. The repo doesn't need to be pre-watched — the hook resolves and
registers the root with the daemon on first use.

## Install

Build and install the hook binary:

```sh
cargo install --path .
```

This installs `git-fsmonitor-watchman-rs` to `~/.cargo/bin` (make sure that's
on `PATH`), or build it manually with `cargo build --release` and point git
at `target/release/git-fsmonitor-watchman-rs`.

Then, in the repo you want to speed up:

```sh
git config core.fsmonitorHookVersion 2
git config core.fsmonitor git-fsmonitor-watchman-rs
```

Use the full path instead of the bare binary name for `core.fsmonitor` if it
isn't on the `PATH` that git's hook execution sees.

## Verifying it works

Run a git command that consults fsmonitor (e.g. `git status`) and it should
behave as before, just faster on repeat runs. To debug the hook in
isolation, invoke it directly with the same two arguments git would pass —
hook version and the last token (empty string for a full scan):

```sh
git-fsmonitor-watchman-rs 2 ""
```

It should print a new clock token followed by a NUL-separated file list.
