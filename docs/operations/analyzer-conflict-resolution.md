# Analyzer Conflict Resolution (Riverpod vs Drift)

## Problem

`riverpod_generator` and `drift_dev` currently compete on incompatible `analyzer` major versions in this project line.

Observed constraints:

- `riverpod_generator 4.0.3` -> `analyzer ^9.0.0`
- `drift_dev 2.32.0` -> `analyzer ^10.0.0`

Because both are `build_runner` generators in the same package, they must resolve in one dependency graph.

## Recommended Approach

### Short-Term (Recommended Now)

- Keep compatible pair:
  - `riverpod_generator 4.0.3`
  - `drift_dev 2.31.x`
- Keep `analyzer 9.x` line in lockfile.
- Continue feature work with this stable baseline.

Why:
- Lowest delivery risk.
- No architecture split required.
- Verified web/runtime behavior remains stable.

### Mid-Term (If Drift 2.32+ is mandatory)

- Split generator responsibilities by package boundary:
  - package A: Drift schema + `drift_dev` generation
  - package B (app): Riverpod generation only
- Consume generated drift artifacts from package A.

Why:
- Dart dependency resolution is package-scoped.
- Different packages can carry different generator/analyzer lines.

Tradeoff:
- Requires monorepo packaging discipline and CI updates.

### Long-Term

- Upgrade when `riverpod_generator` supports `analyzer 10` (or drift line re-aligns).
- Remove temporary pinning and simplify dependency policy.

## Decision Matrix

1. Need fastest delivery now -> stay on `drift_dev 2.31.x`.
2. Need latest drift features immediately -> split generators by package.
3. No urgent need -> wait for upstream alignment and track versions.

## Operational Checklist

- Pin generator versions in `pubspec.yaml`.
- Do not run broad `--major-versions` blindly on generator stacks.
- After dependency edits:
  - `flutter pub get`
  - `dart run build_runner build --delete-conflicting-outputs`
  - `flutter build web`

## Notes

- This is a generator toolchain conflict, not an application runtime bug.
- Fixing it at dependency policy level is safer than forcing analyzer overrides.
