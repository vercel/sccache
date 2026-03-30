# sccache (Vercel fork)

This is a Vercel-maintained fork of [mozilla/sccache](https://github.com/mozilla/sccache). It is maintained as a stack of patches managed with [Graphite](https://graphite.dev), each branch containing a single feature/fix on top of the `upstream` branch.

For full upstream documentation, see the [upstream README](../README.md) or the [upstream repository](https://github.com/mozilla/sccache#readme).

## Branch structure

- **`upstream`** — Tracks `mozilla/sccache` main. This is the base of the stack.
- **`vercel/ci-fixes`** — Bottom of the stack (CI cleanup).
- **`vercel/multilevel-caching`** through **`vercel/fork-readme-ci`** — Stacked feature branches.
- **`main`** — Always points to the top of the stack (same commit as the topmost branch).

## Patches

Each patch is a single Graphite branch with one commit. The stack order (bottom to top):

0. **CI fixes** (`vercel/ci-fixes`) — Removes coverage, benchmarks, snap, FreeBSD, integration-tests, and unsupported cross-compilation targets from upstream CI.

1. **Crate type allow hash** (`vercel/crate-type-allow-hash`) — When `SCCACHE_RUST_CRATE_TYPE_ALLOW_HASH` is set, all crate types become cacheable. The env var value is hashed into the cache key only when unsupported crate types are present.

2. **Rust basedirs** (`vercel/rust-basedirs`) — Strips `SCCACHE_BASEDIR` prefixes from the Rust hash key so cache entries are portable across machines with different checkout paths.

3. **OpenDAL upgrade + Vercel Artifacts backend** (`vercel/opendal-upgrade-artifacts`) — Upgrades opendal via `[patch.crates-io]` ([apache/opendal#7334](https://github.com/apache/opendal/pull/7334)). Adds Vercel Artifacts cache backend with `SCCACHE_VERCEL_ARTIFACTS_TOKEN`, `_ENDPOINT`, `_TEAM_ID`, `_TEAM_SLUG`.

4. **Reflink-based disk cache** (`vercel/file-clone-reflink`) — Cherry-picked from upstream [mozilla/sccache#2640](https://github.com/mozilla/sccache/pull/2640). `SCCACHE_FILE_CLONE=true` stores entries uncompressed and restores via filesystem reflinks. Drop once merged upstream.

5. **File-clone post-write compression** (`vercel/file-clone-compress`) — Adds `SCCACHE_FILE_CLONE_COMPRESS` to compress reflink entries after write.

6. **Directory dep-info hashing** (`vercel/fix-dir-dep-info`) — Fixes [mozilla/sccache#2653](https://github.com/mozilla/sccache/issues/2653). Handles directories in rustc dep-info by recursively hashing contents.

7. **This README + CI/release workflows + version suffix** (`vercel/fork-readme-ci`) — Fork documentation, `vercel-ci.yml`, `vercel-release.yml`, and `-vercel` suffix on `sccache --version`.

Previously the stack included a **Multi-level caching** patch cherry-picked from [mozilla/sccache#2581](https://github.com/mozilla/sccache/pull/2581); that PR landed upstream (merge commit `d11e2e0`) so the fork-specific patch has been dropped.

## Managing the stack

This fork uses [Graphite](https://graphite.dev) to manage the patch stack. Install with `npm i -g @withgraphite/graphite-cli`.

### Viewing the stack

```bash
gt log
```

### Modifying a patch mid-stack

```bash
gt checkout vercel/<branch-to-edit>
# make changes
git add -A && git commit --amend --no-edit
gt restack                              # rebase all branches above
git checkout vercel/fork-readme-ci      # go to top
git branch -f main HEAD                 # point main at top of stack
git push --force-with-lease origin main vercel/<changed-branch> [other affected branches...]
```

### Adding a new patch to the stack

```bash
gt checkout vercel/<branch-below>       # check out where to insert
gt create vercel/<new-branch> -m "commit message"
# make changes
git add -A && git commit -m "description"
gt restack                              # rebase everything above
git checkout vercel/fork-readme-ci
git branch -f main HEAD
git push --force-with-lease origin main vercel/<new-branch> [restacked branches...]
```

Then create a PR:
```bash
gh pr create --base vercel/<branch-below> --head vercel/<new-branch> --title "[N/M] description"
```

And update the base of the PR that was previously on top of `<branch-below>`:
```bash
gh pr edit <pr-number> --base vercel/<new-branch>
```

### Dropping a merged upstream patch

If an upstream PR (e.g., #2581) gets merged:
```bash
gt checkout vercel/<branch-to-drop>
gt move --onto upstream                 # or onto the branch below it
# if the branch is now empty, delete it
gt restack
git checkout vercel/fork-readme-ci
git branch -f main HEAD
```

### Rebasing on latest upstream

```bash
git fetch upstream
git checkout upstream
git reset --hard upstream/main          # update the upstream branch
git push --force-with-lease origin upstream
gt restack                              # rebase entire stack
git checkout vercel/fork-readme-ci
git branch -f main HEAD                 # point main at top
git push --force-with-lease origin --all
```

### After any stack modification

Always finish with:
```bash
git checkout vercel/fork-readme-ci      # top of stack
git branch -f main HEAD                 # sync main
git push --force-with-lease origin main [affected branches...]
```

## CI & Releases

- **`vercel-ci.yml`** — Runs `cargo test` on Ubuntu and a `cargo build --release` smoke-check for `x86_64-pc-windows-msvc` on every push/PR. The Windows job installs Strawberry Perl via `shogo82148/actions-setup-perl` so `openssl-src`'s Perl build script can run.
- **`vercel-release.yml`** — Manual trigger (`workflow_dispatch`) with an optional `version` input (defaults to `1.0.YYYYMMDD`). Bumps `Cargo.toml` on a throwaway commit, pushes tag `v<version>` (not back to `main`), creates an empty GitHub release, then builds release binaries for `x86_64`/`aarch64` Linux musl, `x86_64`/`aarch64` macOS, and `x86_64` Windows. The build matrix is generated via [`mmastrac/mmm-matrix`](https://github.com/mmastrac/mmm-matrix). Assets are named `sccache-<target>.tar.gz` (`.zip` on Windows) and are consumable by `cargo binstall`.

## Installation via `cargo binstall`

Once a release exists, install the latest prebuilt binary with:

```bash
cargo binstall --git https://github.com/vercel/sccache sccache
```

`Cargo.toml` ships `[package.metadata.binstall]` pointing at `https://github.com/vercel/sccache/releases/latest/download/sccache-<target>.{tar.gz,zip}`, so the command above always pulls the most recent release regardless of the `version` field in `Cargo.toml` (which tracks upstream and does not get bumped per release).

## Verification

After modifying the stack, verify with:
```bash
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings -A unknown-lints -A clippy::type_complexity -A clippy::new-without-default
cargo build
cargo test
```

Generate a per-file diff manifest:
```bash
for commit in $(git log --reverse --format="%H" upstream..HEAD); do
  echo "### $(git log --oneline -1 $commit)"
  git diff-tree --no-commit-id --name-status -r $commit
  echo
done
```
