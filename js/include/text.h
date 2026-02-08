#pragma once
#include <stddef.h>
#include <stdint.h>

#include "dtypes/string.h"
#include "parse.h"

#ifdef __cplusplus
extern "C" {
#endif

#define SOURCE_CHARACTER_MAX 0x10FFFF

// https://tc39.es/ecma262/#prod-SourceCharacter
typedef code_point source_character;

typedef struct code_point_at_result {
  code_point cp;
  size_t code_unit_count;
  bool is_unpaired_surrogate;
} code_point_at_result;


typedef struct {
  code_point* data;
  size_t len;
} code_point_seq;

utf16_string* utf16_encode_code_point(const code_point cp);
utf16_string* code_points_to_string(const code_point* text, size_t len);

code_point utf16_surrogate_pair_to_code_point(
    utf16_code_unit lead, utf16_code_unit trail);

code_point_at_result code_point_at(const utf16_string* string, size_t position);

code_point_seq string_to_code_points(const utf16_string* string);

typedef struct {
  bool success;

  union {
    parse_node* parse_tree;

    // TODO: Define a proper syntax error structure
    void* syntax_errors;
  };
} parse_result;

parse_result parse_text_string(
  const utf16_string* string,
  symbol_id goal_symbol
);

parse_result parse_text_code_points(
  const code_point* code_points,
  size_t len,
  symbol_id goal_symbol
);

#ifdef __cplusplus
}
#endif
