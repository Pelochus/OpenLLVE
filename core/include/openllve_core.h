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

// Strategy construction
OpenLlveStrategy* openllve_strategy_new_llie(void);
OpenLlveStrategy* openllve_strategy_new_temporal(void);
void openllve_strategy_free(OpenLlveStrategy* strategy);

// Strategy execution
int32_t openllve_strategy_process(
    OpenLlveStrategy* strategy,
    const float* input_data,
    size_t input_len,
    float* output_data,
    size_t output_len
);

// Optional toppings
EwmaFilter* openllve_ewma_filter_new(float alpha);
int32_t openllve_ewma_filter_apply(
    EwmaFilter* filter,
    const float* input_data,
    size_t input_len,
    float* output_data,
    size_t output_len
);
void openllve_ewma_filter_free(EwmaFilter* filter);

FrameBlendFilter* openllve_blend_filter_new(float beta);
int32_t openllve_blend_filter_apply(
    const FrameBlendFilter* filter,
    const float* raw_data,
    size_t raw_len,
    const float* enhanced_data,
    size_t enhanced_len,
    float* output_data,
    size_t output_len
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
