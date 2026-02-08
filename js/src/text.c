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
