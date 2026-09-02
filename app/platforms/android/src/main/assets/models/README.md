# OpenLLVE Models Documentation

This directory contains the TensorFlow Lite models used for low-light video enhancement in the OpenLLVE application. Below are the details regarding the models:

## Model Formats
- The models are stored in `.tflite` format, which is optimized for mobile and edge devices.
- Models are quantized to **INT8** or **FP16** to ensure efficient execution on hardware accelerators.

## Usage
- Place your model files in this directory.
- Ensure that the model names match the expected names in the application code.
- Refer to the application documentation for details on how to load and utilize these models within the OpenLLVE framework.

## Supported Models
- **Zero-DCE**: A model designed for low-light image enhancement.
- **MBLLEN**: A lightweight model for enhancing video quality in low-light conditions.
- Additional models may be added in the future as the project evolves.

For any questions or contributions regarding the models, please refer to the main project documentation or contact the project maintainers.