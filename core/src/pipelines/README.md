# Pipelines

Main inference families; each reports its `TemporalMode`.

- `llie.rs`: static frame-wise enhancement, `TemporalMode::Stateless`. Optionally
  combined with `EwmaFilter`. With the `model` cargo feature it runs the Zero-DCE
  model through `ModelRunner`; without the feature (or without a model) `process`
  is an identity stub, so the core builds and tests without a LiteRT runtime.
- `temporal.rs`: stateful temporal pipeline, `TemporalMode::Recurrent`,
  optionally combined with `FrameBlendFilter`. EWMA usually does not apply — the
  model already encodes temporal behavior.

Pipeline logic stays separate from post-processing toppings so each pipeline is
composed intentionally rather than hard-wired.
