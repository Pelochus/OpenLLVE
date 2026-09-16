#ifndef OPENLLVE_CORE_H
#define OPENLLVE_CORE_H

#include <stdint.h>
#include <stddef.h>

// Pure C header. Consumers are C runtimes (Kotlin via JNI, Swift via a
// C bridging header); no C++ consumers, so no C++ linkage guard is needed.

// ABI version. Bump when the ABI changes in a breaking way.
uint32_t openllve_abi_version(void);

// Opaque handles
typedef struct OpenLlveStrategy OpenLlveStrategy;
typedef struct OpenLlveMetrics OpenLlveMetrics;
typedef struct EwmaFilter EwmaFilter;
typedef struct FrameBlendFilter FrameBlendFilter;

// Error codes returned by the C ABI. 0 means success.
enum OpenLlveError {
    OPENLLVE_ERROR_OK = 0,
    OPENLLVE_ERROR_NULL_POINTER = 1,
    OPENLLVE_ERROR_INVALID_PARAMETER = 2,
    OPENLLVE_ERROR_BUFFER_DIMENSION_MISMATCH = 3,
    OPENLLVE_ERROR_INTERNAL = 4
};

// Frame buffers are described by width/height/channels plus a stride in
// BYTES per row (a multiple of 4, at least width*channels*4) and a pointer
// to 32-bit float data holding at least stride*height bytes.

// Strategy construction
OpenLlveStrategy* openllve_strategy_new_llie(void);
OpenLlveStrategy* openllve_strategy_new_temporal(void);
// LLIE with the Zero-DCE model loaded. Returns NULL if the `model` cargo
// feature is not enabled, the TFLite library/model cannot be loaded, or
// model_path is NULL / num_threads is not positive.
OpenLlveStrategy* openllve_strategy_new_llie_with_model(const char* model_path, int num_threads);
void openllve_strategy_free(OpenLlveStrategy* strategy);

// Strategy execution. Input and output frames are described independently
// (a model may change the channel count, e.g. Zero-DCE maps 4 channels to 24).
int32_t openllve_process_frame(
    OpenLlveStrategy* strategy,
    uint32_t in_width,
    uint32_t in_height,
    uint32_t in_channels,
    size_t in_stride,
    const float* in_data,
    uint32_t out_width,
    uint32_t out_height,
    uint32_t out_channels,
    size_t out_stride,
    float* out_data
);

// Optional toppings
EwmaFilter* openllve_ewma_filter_new(float alpha);
int32_t openllve_ewma_filter_apply(
    EwmaFilter* filter,
    uint32_t width,
    uint32_t height,
    uint32_t channels,
    size_t stride,
    const float* input_data,
    float* output_data
);
void openllve_ewma_filter_free(EwmaFilter* filter);

FrameBlendFilter* openllve_blend_filter_new(float beta);
int32_t openllve_blend_filter_apply(
    const FrameBlendFilter* filter,
    uint32_t width,
    uint32_t height,
    uint32_t channels,
    size_t stride,
    const float* raw_data,
    const float* enhanced_data,
    float* output_data
);
void openllve_blend_filter_free(FrameBlendFilter* filter);

// Metrics collection
OpenLlveMetrics* openllve_metrics_new(void);
OpenLlveMetrics* openllve_metrics_new_with_warmup(size_t warmup_frames);
void openllve_metrics_record(OpenLlveMetrics* metrics, double latency_ms);
double openllve_metrics_get_average(const OpenLlveMetrics* metrics);
double openllve_metrics_get_median(const OpenLlveMetrics* metrics);
double openllve_metrics_get_fps(const OpenLlveMetrics* metrics);
double openllve_metrics_get_p99(const OpenLlveMetrics* metrics);
size_t openllve_metrics_get_count(const OpenLlveMetrics* metrics);
void openllve_metrics_reset(OpenLlveMetrics* metrics);
void openllve_metrics_free(OpenLlveMetrics* metrics);

#endif // OPENLLVE_CORE_H
