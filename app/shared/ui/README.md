# Shared UI

This folder holds view state, view model, and presentation logic that can be shared between platforms where appropriate.

## Purpose

- UI state models
- shared data contracts
- presentation-level logic
- abstraction for host-specific rendering

## Rule

Keep this layer free of direct Android or iOS runtime references unless behind clearly isolated adapters.
