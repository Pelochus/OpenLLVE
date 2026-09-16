# Strategies

This module defines the main inference families for OpenLLVE.

- `llie.rs`: static frame-wise enhancement, optionally combined with `EwmaFilter`.
  With the `model` feature enabled, it runs the Zero-DCE model (`ModelRunner`
  from ADR-0001): frame in → enhanced frame out. Without the feature (or without
  a model) `process` is an identity stub so the core still builds and tests
  without the TFLite runtime present.
- `temporal.rs`: stateful temporal strategy, optionally combined with
  `FrameBlendFilter`.

Keep core strategy logic separated from post-processing toppings so each
pipeline can be composed intentionally.
