#ifndef OPENLLVE_CORE_H
#define OPENLLVE_CORE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handles
typedef struct OpenLlveStrategy OpenLlveStrategy;
typedef struct OpenLlveMetrics OpenLlveMetrics;
typedef struct EwmaFilter EwmaFilter;
typedef struct FrameBlendFilter FrameBlendFilter;

// Strategy construction
OpenLlveStrategy* openllve_strategy_new_llie(void);
OpenLlveStrategy* openllve_strategy_new_temporal(void);
void openllve_strategy_free(OpenLlveStrategy* strategy);

// Strategy execution
bool openllve_strategy_process(
    OpenLlveStrategy* strategy,
    const float* input_data,
    size_t input_len,
    float* output_data,
    size_t output_len
);

// Optional toppings
EwmaFilter* openllve_ewma_filter_new(float alpha);
bool openllve_ewma_filter_apply(
    EwmaFilter* filter,
    const float* input_data,
    size_t input_len,
    float* output_data,
    size_t output_len
);
void openllve_ewma_filter_free(EwmaFilter* filter);

FrameBlendFilter* openllve_blend_filter_new(float beta);
bool openllve_blend_filter_apply(
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
void openllve_metrics_record(OpenLlveMetrics* metrics, float latency_ms);
float openllve_metrics_get_average(const OpenLlveMetrics* metrics);
float openllve_metrics_get_fps(const OpenLlveMetrics* metrics);
float openllve_metrics_get_p99(const OpenLlveMetrics* metrics);
void openllve_metrics_reset(OpenLlveMetrics* metrics);
void openllve_metrics_free(OpenLlveMetrics* metrics);

#ifdef __cplusplus
}
#endif

#endif // OPENLLVE_CORE_H
