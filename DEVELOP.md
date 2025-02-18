# Develop (with) Pixlie AI

This guide is intended for developers who want to contribute to Pixlie AI or are interested in the technical details of
the project.

## Software needed

- Linux, MacOS, or Windows (WSL2)
- Git
- Rust (we install with rustup)
- Node.js (we use LTS)
- pnpm

## Prerequisites

- Install Git, Rust, Python, Node.js, pnpm
- Install Clang (needed for RocksDB library)
- Clone this repository with git

_Note_: If you are using Windows, we suggest using WSL2 which is what we plan to support.

## Run the web UI

The web UI is a SolidJS app. It is in the `admin` directory.
Run the following commands to start the web UI:

```bash
cd admin
pnpm install
pnpm run dev
```

The web UI is available at http://localhost:5173

## Run the backend

The backend is a Rust app. It is in the `pixlie_ai` directory.
Using a separate terminal, run the following commands to start the backend:

```bash
cd pixlie_ai
RUST_LOG=debug cargo run --bin cli
```

Change the `RUST_LOG` to `info` or `debug` depending on the level of logging you want.
