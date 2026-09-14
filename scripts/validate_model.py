"""Validate a TensorFlow Lite model file.

Loads the model with the TFLite Python interpreter and prints its input and
output tensor details. Exits non-zero if the model cannot be loaded.
"""

import sys

import tensorflow as tf


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_model.py <model.tflite>", file=sys.stderr)
        return 2

    model_path = sys.argv[1]
    interpreter = tf.lite.interpreter.Interpreter(model_path=model_path)
    input_details = interpreter.get_input_details()
    output_details = interpreter.get_output_details()
    interpreter.allocate_tensors()
    print("Model input details:", input_details)
    print("Model output details:", output_details)
    return 0


if __name__ == "__main__":
    sys.exit(main())
