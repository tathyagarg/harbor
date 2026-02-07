#pragma once
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct js_context js_context;

js_context *js_create_context();
void js_destroy_context(js_context *ctx);

#ifdef __cplusplus
}
#endif
