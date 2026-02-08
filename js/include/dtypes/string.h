#pragma once
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>
#include <assert.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LEADING_SURROGATE_MIN 0xD800
#define LEADING_SURROGATE_MAX 0xDBFF

#define TRAILING_SURROGATE_MIN 0xDC00
#define TRAILING_SURROGATE_MAX 0xDFFF

#define IN_INCLUSIVE_RANGE(cu, min, max) ((min) <= (cu) && (cu) <= (max))

typedef uint16_t utf16_code_unit;

typedef uint32_t code_point;

typedef utf16_code_unit* _shadow_utf16_string;

typedef struct {
  _shadow_utf16_string data;
  size_t len;
} utf16_string;

// NOTE: Result is stored on heap and must be freed by the caller.
utf16_string* string_with_size(size_t n) {
  _shadow_utf16_string data = (_shadow_utf16_string)
    (malloc(n * sizeof(utf16_code_unit)));

  if (!data) {
    return NULL;
  }

  utf16_string* result = malloc(sizeof(utf16_string));

  result->data = data;
  result->len = n;

  return result;
}

void free_string(utf16_string* s) {
  free(s->data);
  free(s);
}

bool is_leading_surrogate(utf16_code_unit cu) {
  return IN_INCLUSIVE_RANGE(cu, LEADING_SURROGATE_MIN, LEADING_SURROGATE_MAX);
}

bool is_trailing_surrogate(utf16_code_unit cu) {
  return IN_INCLUSIVE_RANGE(cu, TRAILING_SURROGATE_MIN, TRAILING_SURROGATE_MAX);
}

#ifdef __cplusplus
}
#endif
