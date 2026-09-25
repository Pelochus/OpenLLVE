# Shared Layer

Kotlin Multiplatform module (`:shared`) with app-neutral logic shared by
Android and (later) iOS:

- `domain/` — contracts: `ComputeTarget`, `EnhancementSettings`, `MediaInput`,
  `BackendSelection`, `ProcessingMetrics`, `EnhancementEngine`, `SettingsStore`
- `media/` — `FrameImage`, `FramePixels`, `VideoMetadata`
- `ui/` — `UiState`

Android consumes it via `implementation(project(":shared"))` with thin
boundary adapters (`BitmapFrameAdapter`, `MediaInput.sourceUri()`).

Rule: keep here only logic not tied to Android or iOS runtime APIs;
performance-sensitive compute stays in the `core/` crate.
