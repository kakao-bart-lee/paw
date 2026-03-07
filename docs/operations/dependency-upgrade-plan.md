# Dependency Upgrade Plan

## Objective

Stabilize current web/runtime behavior first, then upgrade dependencies in low-risk phases.

## Current Verified Baseline

- `flutter_rust_bridge` runtime/codegen/generated code aligned at `2.11.1`.
- `flutter build web` succeeds.
- Web DB path is split by platform import (`drift/web` for web, `drift/native` for native).

## Upgrade Strategy

1. Lock critical runtime packages first.
2. Upgrade non-breaking patch/minor dependencies.
3. Upgrade one major package group at a time.
4. Run smoke checks after each group.
5. Keep rollback points as small commits.

## Recommended Order

1. Tooling only: `flutter_lints`, `build_runner`, `json_serializable`, `drift_dev`.
2. Routing/state group: `go_router`, `flutter_riverpod`, `riverpod_annotation`, `riverpod_generator`.
3. Platform/security group: `flutter_secure_storage` and platform packages.
4. Data/storage group: `drift`, `sqlite3`, `sqlparser`.

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
