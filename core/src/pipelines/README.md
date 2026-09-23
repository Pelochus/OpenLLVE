# Pipelines

This module defines the main inference families for OpenLLVE.

- `llie.rs`: static frame-wise enhancement, optionally combined with
  `EwmaFilter`. With the `model` feature enabled, it runs the Zero-DCE model
  (`ModelRunner`, see `docs/architecture/ARCHITECTURE.md` §11): frame in → enhanced frame out. Without the
  feature (or without a model) `process` is an identity stub so the core still
  builds and tests without the LiteRT runtime present.
- `temporal.rs`: stateful temporal pipeline, optionally combined with
  `FrameBlendFilter`.

Each pipeline reports its [`TemporalMode`]: `LliePipeline` is `Stateless`
(frame-wise) and `LlveTemporalPipeline` is `Recurrent` (carries inter-frame
state).

Keep core pipeline logic separated from post-processing toppings so each
pipeline can be composed intentionally.
