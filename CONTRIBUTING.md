# Contributing

1. Install Rust (rustup), Node.js and pnpm (for the lint only), then run `pnpm install`.
2. Make one logical change per commit; the message says what and why.
3. Run `pnpm lint` and `cargo test --workspace` before opening a pull request.
4. Verify in the real app: run the bridge with Dolphin, or `cargo run -p tools --bin dsu_probe` while moving a controller.
5. Keep the README and `docs/how-it-works.md` in step with behaviour changes.

Never commit personal data: account names, device serials, IP addresses, logs.
Never kill processes by name; use `scripts/stop.ps1`.
