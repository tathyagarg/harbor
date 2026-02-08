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

code_point utf16_surrogate_pair_to_code_point(
    utf16_code_unit lead, utf16_code_unit trail) {
  assert(is_leading_surrogate(lead));
  assert(is_trailing_surrogate(trail));

  code_point cp = (lead - LEADING_SURROGATE_MIN) * 0x400 + 
    (trail - TRAILING_SURROGATE_MIN) + 0x10000;

  return cp;
}

typedef struct code_point_at_result {
  code_point cp;
  size_t code_unit_count;
  bool is_unpaired_surrogate;
} code_point_at_result;

code_point_at_result code_point_at(utf16_string* string, size_t position) {
  size_t size = string->len;

  assert(position >= 0 && position < size);

  utf16_code_unit first = string->data[position];
  code_point cp = (code_point)first;

  if (!is_leading_surrogate(first) && !is_trailing_surrogate(first)) {
    return (code_point_at_result){
      .cp = cp,
      .code_unit_count = 1,
      .is_unpaired_surrogate = false
    };
  }

  if (is_trailing_surrogate(first) || position + 1 == size) {
    return (code_point_at_result){
      .cp = cp,
      .code_unit_count = 1,
      .is_unpaired_surrogate = true
    };
  }

  utf16_code_unit second = string->data[position + 1];

  if (!is_trailing_surrogate(second)) {
    return (code_point_at_result){
      .cp = cp,
      .code_unit_count = 1,
      .is_unpaired_surrogate = true
    };
  }

  cp = utf16_surrogate_pair_to_code_point(first, second);

  return (code_point_at_result){
    .cp = cp,
    .code_unit_count = 2,
    .is_unpaired_surrogate = false
  };
}

// NOTE: Result is stored on heap and must be freed by the caller.
code_point* string_to_code_points(const utf16_string* string) {
  size_t cps_len = 0;
  code_point* cps = (code_point*)malloc(string->len * sizeof(code_point));

  for (size_t i = 0; i < string->len; i++) {
    code_point_at_result result = code_point_at(
        (utf16_string*)string, i);

    cps[cps_len++] = result.cp;
    i += result.code_unit_count - 1;
  }

  cps = (code_point*)realloc(cps, cps_len * sizeof(code_point));

  return cps;
}

#ifdef __cplusplus
}
#endif
