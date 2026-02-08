#include "text.h"

#include <stdlib.h>
#include <assert.h>
#include <math.h>
#include <string.h>

// Encodes a Unicode code point into a UTF-16 string.
// NOTE: Result is stored on heap and must be freed by the caller.
// Characters are NOT null-terminated.
utf16_string* utf16_encode_code_point(source_character cp) {
  assert(0 <= cp && cp <= SOURCE_CHARACTER_MAX);

  if (cp <= 0xFFFF) {
    utf16_code_unit unit = (utf16_code_unit)(cp & 0xFFFF);

    utf16_string* result = string_with_size(1);
    if (result) {
      result->data[0] = unit;
      return result;
    } else {
      return NULL;
    }
  }

  utf16_code_unit cu1 = floor((float)(cp - 0x10000) / 0x400) + 0xD800;
  utf16_code_unit cu2 = ((cp - 0x10000) % 0x400) + 0xDC00;

  utf16_string* result = string_with_size(2);
  if (result) {
    result->data[0] = cu1;
    result->data[1] = cu2;
    return result;
  } else {
    return NULL;
  }
}

utf16_string* code_points_to_string(const source_character *text, size_t len) {
  size_t length = 0;
  
  utf16_string **cps = (utf16_string**)malloc(len * sizeof(utf16_string*));
  for (size_t i = 0; i < len; i++) {
    utf16_string* ch = utf16_encode_code_point(text[i]);

    cps[i] = ch;
    length += ch->len;
  }

  utf16_string* result = string_with_size(length);

  size_t copied = 0;
  for (size_t i = 0; i < len; i++) {
    memcpy(result->data + copied, cps[i]->data, cps[i]->len);
    copied += cps[i]->len;
    free_string(cps[i]);
  }

  return result;
}

code_point utf16_surrogate_pair_to_code_point(
    utf16_code_unit lead, utf16_code_unit trail) {
  assert(is_leading_surrogate(lead));
  assert(is_trailing_surrogate(trail));

  code_point cp = (lead - LEADING_SURROGATE_MIN) * 0x400 + 
    (trail - TRAILING_SURROGATE_MIN) + 0x10000;

  return cp;
}

code_point_at_result code_point_at(
  const utf16_string* string, size_t position) {
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
code_point_seq string_to_code_points(const utf16_string* string) {
  size_t cps_len = 0;
  code_point* cps = (code_point*)malloc(string->len * sizeof(code_point));

  for (size_t i = 0; i < string->len; i++) {
    code_point_at_result result = code_point_at(
        (utf16_string*)string, i);

    cps[cps_len++] = result.cp;
    i += result.code_unit_count - 1;
  }

  cps = (code_point*)realloc(cps, cps_len * sizeof(code_point));

  return (code_point_seq){
    .data = cps,
    .len = cps_len
  };
}

parse_result parse_text_string(
  const utf16_string* string,
  symbol_id goal_symbol
) {
  code_point_seq cps = string_to_code_points(string);

  parse_result result = parse_text_code_points(
    cps.data,
    cps.len,
    goal_symbol
  );

  free(cps.data);

  return result;
}

parse_result parse_text_code_points(
  const code_point* code_points,
  size_t len,
  symbol_id goal_symbol
) {
  switch (goal_symbol) {}

  return (parse_result){
    .success = false,
    .syntax_errors = NULL
  };
}
