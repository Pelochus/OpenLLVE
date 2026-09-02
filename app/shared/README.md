# Shared Layer

This directory holds app-neutral logic for the product layer.

## Contents

- `benchmarking/`: benchmark configuration, metrics aggregation, and shared performance scenarios
- `ui/`: presentation contracts, state models, and cross-platform UI abstractions
- `core/`: shared domain logic that should be KMP-friendly and platform-agnostic

## Rule

Keep here only logic that is not tied to Android or iOS runtime APIs. All shared app logic should prefer Kotlin Multiplatform, while performance-sensitive algorithmic compute remains in the sibling `core/` crate, not under `app/shared`.
