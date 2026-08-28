# rubar

Barcode encoding and rendering with exact module-level geometry. Rust core,
Python bindings via PyO3/maturin.

## Layout

- `crates/rubar-core` — pure Rust: encoding, geometry, rendering. No PyO3.
  This is the crate published to crates.io, and what rupdf depends on.
- `crates/rubar-py` — thin PyO3 wrapper (`cdylib` only, **not** publishable to
  crates.io). Published to PyPI as `rubar`.
- `python/rubar` — Python source and type stubs, packaged by maturin.

## Testing

```bash
cargo test -p rubar-core        # include doc tests; --lib alone skips them
.venv/bin/python -m pytest python/tests -q
```

Run pytest through `.venv/bin/python` directly. `uv run pytest` may resolve a
different interpreter than the one `maturin develop` built the extension for,
which shows up as a confusing `No module named 'rubar._rubar'`.

Rebuild the extension after any Rust change: `uv run maturin develop --uv`.

## Releasing

Follow the `/release` skill. Everything below is where this repo differs from
what that skill assumes — read it first or you will do the wrong thing twice.

**No CHANGELOG.md, deliberately.** The skill's prepare-release step will offer
to create one. Don't. Release notes are written directly into the GitHub
release, structured as `## Highlights` / `## Wheels`, and the wheels are
attached to that release as assets. Match the previous release's shape:
`gh release view v0.3.0`.

**Two registries, one version.** The workspace version in the root
`Cargo.toml` drives both crates; `pyproject.toml` carries the same number
independently, so bump both. Then:

```bash
cargo publish -p rubar-core     # crates.io — the rlib only
twine upload target/wheels/rubar-VERSION*
```

`cargo publish` without `-p rubar-core` also tries `rubar-py`, which is
`cdylib`-only and cannot be consumed as a Rust dependency. It fails during
tarball verification with a bare `clang: linker command failed` — that error
means "you forgot `-p`", not that anything is broken.

**Credentials** live in `.env` (loaded by direnv): `CARGO_REGISTRY_TOKEN` and
`MATURIN_PYPI_TOKEN`. There is also a `~/.pypirc` that twine picks up.

**Wheel matrix** is Python 3.11, 3.12, 3.13 × macOS arm64, manylinux x86_64,
manylinux aarch64 — 9 wheels plus an sdist, 10 artifacts. The `classifiers`
and `requires-python` in `pyproject.toml` are the source of truth for the
Python versions; keep them accurate rather than trusting this paragraph, and
cross-check against what actually shipped last time:

```bash
curl -s https://pypi.org/pypi/rubar/json | python3 -c "import json,sys; [print(u['filename']) for u in json.load(sys.stdin)['urls']]"
```

Do not take the previous *rubar* release as the target on its own — 0.2.0
shipped only 5 files, which looked like policy but was an incomplete build.
A partial matrix silently drops platforms that consumers resolve against.

**Verify the artifact, not just the source.** Install a built wheel into a
clean venv and check real output before uploading — a stale `target/wheels`
entry is easy to ship by accident:

```bash
python3.12 -m venv /tmp/verify && /tmp/verify/bin/pip install target/wheels/rubar-VERSION-cp312-cp312-macosx_11_0_arm64.whl
```

**Verify uploads via the PyPI JSON API**, not twine's stdout — its progress UI
overwrites long aarch64 filenames, so the list it prints is unreliable:

```bash
curl -s https://pypi.org/pypi/rubar/VERSION/json | python3 -c "import json,sys; d=json.load(sys.stdin); print(len(d['urls']))"
```

**Downstream chain.** rubar → rupdf (bumps `rubar-core`, ships its own wheels)
→ sklib (bumps its rupdf pin). Pre-1.0 caret semantics mean a minor bump is a
real edit in rupdf's `Cargo.toml`, not a `cargo update`. Releasing rubar does
not oblige anyone downstream to release; that is Lee's call each time.

## Code 128 encoding

`encode_code128` plans code sets with a DP to minimise symbol count — digit
runs pack two-per-symbol into Code Set C. Callers size barcodes from the
symbol count, so a suboptimal encoding draws bars finer than intended without
erroring. If you touch the planner, the parity test against sklib's
`min_symbol_count` oracle in `encode/code128.rs` is the thing that will catch
you; regenerate its table from the oracle rather than editing rows by hand.

The planner cannot emit SHIFT — the `barcoders` backend has no shift state.
See the comment on `cost_table` for the bounded consequence.
