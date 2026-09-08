# Filters

This module holds optional post-processing toppings that can be attached to a strategy.

- `ewma.rs`: temporal anti-flicker smoothing for LLIE-style pipelines
- `blend.rs`: raw/input + output blending for stable mixing

Prefer using these as optional toppings instead of hard-wiring them into every strategy.
