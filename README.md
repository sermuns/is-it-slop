# `is-it-slop`

<p align=center>

![your ai slop bores me](./media/0d5.jpg)

</p>

In my experience, low effort LLM-generated slop Rust projects have three common hallmarks:

- using Rust 2021
- (if using workspaces) using workspace resolver version 2
- using generally outdated versions of dependencies

This simple command-line tool can check for this.

## Usage

```present cargo run -- -h
CLI for detecting slop smell

Usage: is-it-slop [OPTIONS] <GITHUB_PROJECT_OR_URL>

Arguments:
  <GITHUB_PROJECT_OR_URL>  Either <USER>/<REPO> or full URL

Options:
      --check              Emit non-zero exit code if any slop detected
      --git-ref <GIT_REF>  HEAD is a reasonable standard, but you can manually specify branch name or specific commit [default: HEAD]
  -h, --help               Print help
  -V, --version            Print version
```

## TODO

I want to detect more common hallmarks and avoid false positives.

1. There are _of course_ non-slop reasons for using older versions, like maybe the project was literally made at a time when that was the latest, maybe some regression in dependency is forcing them to stay?

2. Scanning commit history. Not sure exactly what could qualify as "sloppy". Massive commits? Em-dashes in the message? `coauthored-by` some LLM?

---

**No artificial intelligence was used in the making of this.**

<a href="https://brainmade.org/">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://brainmade.org/black-logo.svg">
  <img alt="brainmade" src="https://brainmade.org/white-logo.svg">
</picture>
</a>
