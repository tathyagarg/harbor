#include "js.h"
#include <stdio.h>
#include <stdlib.h>

struct js_context {
  int placeholder;
};

js_context *js_create_context() {
  js_context *ctx = (js_context *)malloc(sizeof(js_context));
  if (ctx) {
    ctx->placeholder = 0;
  }

  printf("JavaScript context created.\n");

  return ctx;
}

void js_destroy_context(js_context *ctx) {
  if (ctx) {
    free(ctx);
  }
}
