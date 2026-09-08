# iOS Platform Layer

This directory is the future iOS host implementation for OpenLLVE.

## Purpose

- AVFoundation capture pipeline
- SwiftUI or UIKit view layer
- CoreML / MPS integration
- iOS runtime lifecycle and adapters

## Rule

Re-use shared domain logic and benchmarking contracts here instead of duplicating the compute layer.
