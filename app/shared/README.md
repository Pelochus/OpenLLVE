# Shared Layer

Kotlin Multiplatform module (`:shared`) — app logic shared by Android and
(later) iOS, with no host SDK APIs.

- `domain/` — `BackendSelection`, `ComputeTarget`, `EnhancementEngine`,
  `EnhancementSettings`, `MediaInput`, `ProcessingMetrics`, `SettingsStore`
- `media/` — `FrameImage`, `FramePixels`, `VideoMetadata`
- `ui/` — `UiState`
- `log/` — `LogBuffer` ring buffer, `LogEntry`, `LogLevel`, `LogLock`
  (+ `androidMain/`, `iosMain/` mirrors)

Android consumes it with `implementation(project(":shared"))` and thin
boundary adapters (`BitmapFrameAdapter`, `MediaInput.sourceUri()`).

Rule: keep here only logic not tied to a host runtime API; performance-sensitive
compute stays in the Rust `core/` crate.
