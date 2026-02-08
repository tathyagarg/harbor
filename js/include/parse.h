#pragma once
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint16_t symbol_id;

typedef struct {
  symbol_id id;
  bool is_terminal;
} symbol;

typedef struct {
  symbol_id lhs;
  size_t production_length;
  symbol_id *rhs;
} production;

typedef struct parse_node {
  symbol_id symbol;

  size_t start_cp_idx;
  size_t end_cp_idx;

  const production *prod;

  // NOTE: prod->production_length == len(children)
  struct parse_node **children;
} parse_node;

#ifdef __cplusplus
}
#endif
