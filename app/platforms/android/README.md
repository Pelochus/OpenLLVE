# Android Platform Layer

This directory contains Android-specific runtime code for the app.

## Purpose

- CameraX / camera capture
- Compose UI and screen composition
- Android lifecycle management
- device-specific accelerator integration
- bridge code into the shared abstractions and Rust core

## Rule

Keep Android-specific behavior confined here so the shared logic remains portable. The actual Android source tree lives under `app/platforms/android/src`, while `app/shared` remains the KMP/shared app boundary.
