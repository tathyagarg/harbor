#include "lex.h"

lex_token* inp_elem_hashbang_or_reg_exp_to_tokens(
  const code_point_seq* code_points) {

}

lex_token* text_to_tokens(
  const code_point_seq* code_points,
  lexical_goal_symbol goal_symbol
) {
  switch (goal_symbol) {
    case GOAL_INP_ELEM_HASHBANG_OR_REG_EXP: {
      return inp_elem_hashbang_or_reg_exp_to_tokens(code_points);
    }
  }
}
