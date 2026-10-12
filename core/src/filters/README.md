# Filters

Optional post-processing toppings, attached to a strategy rather than hard-wired
into every pipeline.

- `ewma.rs` — `EwmaFilter`: anti-flicker smoothing,
  `out = alpha * current + (1 - alpha) * previous` (`alpha` in `[0, 1]`; the app
  runs 0.8, weighting the current frame so ghosting stays invisible on video)
- `blend.rs` — `FrameBlendFilter`: raw-vs-enhanced mixing,
  `output = beta * enhanced + (1 - beta) * raw_input` (`beta` in `[0, 1]`; the
  app runs 0.9, keeping mostly model output)

Weights are validated at construction. The app supplies them as `CoreToppings`
constants; promoting them to `EnhancementSettings` values is open work.
