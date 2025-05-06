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

## API Documentation

We have 3 API documentation available when running Pixlie locally:

- `/swagger`  - chokes on large responses quite bad.
- `/redoc` - doesn't have the 'Try it out’ feature.
- `/rapidoc` - It does not have reported choking issues and documentation & nav are quite good. But due to some bug, {project_id} is not showing up as an input parameter, so effectively the engine endpoints are currently unusable.

## Releasing a new version

To release a new version, you need to follow these steps:

1. Update the version number `X.Y.Z` in the `VERSION` file in repo root.
2. `pixlie_ai`
    - Update the version in `Cargo.toml` and `src/lib.rs`.
    - Run `cargo test` to ensure everything is fine.
3. `admin`
    - Update the version in `admin/package.json`.
    - Run `pnpm version:check` to ensure everything is fine.
    - Run `pnpm build` to check for a successful build.
4. After merging these changes to `main`, create a new release tag:
    - `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
    - or `git tag -as vX.Y.Z -m "Release vX.Y.Z"` if you want to sign your tag.
    - or `git tag -a vX.Y.Z` if you want to add a multiline release message.
5. Push the tag to the remote repository:
    - `git push origin vX.Y.Z`
6. If everything is fine and passes through the build & release process on Github Actions, the release will be published on https://github.com/pixlie/PixlieAI/releases

> P.S. There is no check whether the tagged commit SHA is a part of the main branch. But we should ensure that we only release commits from the main branch.
