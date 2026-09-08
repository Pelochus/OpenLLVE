# Strategies

This module defines the main inference families for OpenLLVE.

- `llie.rs`: static frame-wise enhancement, optionally combined with `EwmaFilter`
- `temporal.rs`: stateful temporal strategy, optionally combined with `FrameBlendFilter`

Keep core strategy logic separated from post-processing toppings so each pipeline can be composed intentionally.
