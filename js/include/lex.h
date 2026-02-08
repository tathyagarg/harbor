#pragma once
#include <stddef.h>

#include "text.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  GOAL_INP_ELEM_HASHBANG_OR_REG_EXP,

  // TODO: Add the rest:
  // https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
} lexical_goal_symbol;

typedef enum {
  TOKEN_WHITESPACE,
  TOKEN_LINE_TERMINATOR,
  TOKEN_COMMENT,
  TOKEN_COMMON_TOKEN,
  TOKEN_HASHBANG_COMMENT,
  TOKEN_REGULAR_EXPRESSION_LITERAL,
} lex_token_type;

typedef struct {
  lex_token_type type;

  union {

  } data;
} lex_token;

lex_token* text_to_tokens(
  const code_point_seq* code_points,
  lexical_goal_symbol goal_symbol
);

#ifdef __cplusplus
}
#endif
