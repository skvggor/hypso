<p align="center"><img src="assets/icons/icon-256.png" width="96" alt="Hypso logo"></p>

# Hypso

Native desktop app that generates 4K **topographic-style wallpapers** — fractal
contour-map line art over a flat background, with reserved zones for typography.
Rust + [Slint](https://slint.dev) for the GUI,
[resvg](https://github.com/linebender/resvg) / tiny-skia for rasterization.
Single binary for Linux and Windows.

## Status

Early development. The generative core is built test-first (red → green →
refactor); see `openspec/changes/topographic-wallpaper-generator/` for the
proposal, design, specs, and task list.

## Commands

```sh
cargo run --release            # run the app
cargo test --lib               # library tests (the generative core)
cargo fmt --all --check        # formatting gate
cargo clippy --all-targets -- -D warnings   # lint gate
cargo llvm-cov --lib --fail-under-lines 80  # coverage gate (library ≥ 80%)
```

## License

MIT — see [LICENSE](LICENSE). Bundled Montserrat font under the SIL Open Font
License (`assets/fonts/OFL.txt`).
