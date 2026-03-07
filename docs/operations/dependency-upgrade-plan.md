# Dependency Upgrade Plan

## Objective

Stabilize current web/runtime behavior first, then upgrade dependencies in low-risk phases.

## Current Verified Baseline

- `flutter_rust_bridge` runtime/codegen/generated code aligned at `2.11.1`.
- `flutter build web` succeeds.
- Web DB path is split by platform import (`drift/web` for web, `drift/native` for native).
- Riverpod migrated to v3/v4 style (`Notifier` + family constructor pattern).
- `go_router`, `get_it`, `flutter_secure_storage` major upgrades applied and web build verified.
- `drift`/`drift_dev` are pinned at `2.31.x` due to analyzer compatibility.

## Upgrade Strategy

1. Lock critical runtime packages first.
2. Upgrade non-breaking patch/minor dependencies.
3. Upgrade one major package group at a time.
4. Run smoke checks after each group.
5. Keep rollback points as small commits.

## Recommended Order

1. Completed: Tooling + state/router/platform (`flutter_lints`, `build_runner`, `json_serializable`, `drift_dev`, `riverpod`, `go_router`, `get_it`, `flutter_secure_storage`).
2. Next: Data/storage group (`drift`, `sqlite3`, `sqlparser`) full latest.
Blocked until `riverpod_generator` supports `analyzer ^10` (or project migrates generator stack accordingly).
3. Next: `intl` and remaining transitive updates with strict smoke checks.

## Safety Checklist

- Ensure `flutter_rust_bridge` runtime and generated code versions are identical.
- Regenerate FRB bindings after FRB-related changes.
- Keep web path free from `dart:io` and native sqlite imports.
- Validate at least:
  - `flutter build web`
  - `cargo build -p paw-server`
  - local startup script (`./scripts/run-local-stack.sh`)
- Do not batch unrelated major upgrades in one commit.

## Command Set

```bash
# inspect outdated packages
cd paw-client
flutter pub outdated

# selective upgrades (repeat by package group)
flutter pub upgrade <package_name>

# FRB regenerate when needed
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml

# web smoke
flutter build web
```

## Rollback Rule

- If a package group fails smoke checks, revert that group commit only.
- Keep the previous verified baseline deployable at all times.

## Known Blockers

- `drift_dev >= 2.32.0` requires `analyzer ^10`.
- Current `riverpod_generator 4.0.3` requires `analyzer ^9`.
- Therefore `drift_dev 2.32.x` and `riverpod_generator 4.0.3` cannot coexist in one lockfile.
- Resolution guidance is documented in `docs/operations/analyzer-conflict-resolution.md`.
