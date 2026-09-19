# Shared Layer

This directory holds app-neutral logic for the product layer, implemented as a
Kotlin Multiplatform module (`:shared`) so the same code compiles for Android
and (later) iOS.

## Module layout

```text
app/shared/
├── build.gradle.kts                  # KMP: androidTarget + iOS targets (macOS hosts)
└── src/commonMain/kotlin/openllve/shared/
    ├── domain/                       # shared domain contracts
    │   ├── ComputeTarget.kt          # backend enum (CPU/XNNPACK/GPU/NPU)
    │   ├── EnhancementSettings.kt    # user-facing config (compute target + toppings)
    │   ├── MediaInput.kt             # image/video input (platform-neutral URI string)
    │   ├── BackendSelection.kt       # requested vs actual backend + fallback reason
    │   ├── ProcessingMetrics.kt      # latency/throughput metrics for a run
    │   └── EnhancementEngine.kt      # UI<->engine seam (no host types in the interface)
    ├── media/                        # shared frame/pixel abstractions
    │   ├── FrameImage.kt             # platform-neutral decoded frame (ARGB_8888 ints)
    │   ├── FramePixels.kt            # ARGB <-> RGB-float conversion (pure math)
    │   └── VideoMetadata.kt          # basic video metadata record
    └── ui/
        └── UiState.kt                # flat presentation state for the enhancement flow
```

The Android platform consumes it via `implementation(project(":shared"))` and
adds thin boundary adapters (`openllve.android.media.BitmapFrameAdapter`,
`MediaInput.sourceUri()`) where native types (`Bitmap`, `android.net.Uri`)
meet the shared contracts.

## Subfolder READMEs

- `benchmarking/`: benchmark configuration, metrics aggregation, and shared
  performance scenarios (code lands here as `openllve.shared.benchmarking`
  when P3.6 `BenchmarkRun` lands)
- `ui/`: presentation contracts, state models, and cross-platform UI
  abstractions

## Rule

Keep here only logic that is not tied to Android or iOS runtime APIs. All
shared app logic should prefer Kotlin Multiplatform, while performance-sensitive
algorithmic compute remains in the sibling `core/` crate, not under
`app/shared`.
