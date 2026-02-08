#pragma once
#include <stddef.h>
#include <stdint.h>

#include "ecma/string.h"

#ifdef __cplusplus
extern "C" {
#endif

#define SOURCE_CHARACTER_MAX 0x10FFFF

// https://tc39.es/ecma262/#prod-SourceCharacter
typedef code_point source_character;

utf16_string* utf16_encode_code_point(const code_point cp);
utf16_string* code_points_to_string(const code_point* text, size_t len);

#ifdef __cplusplus
}
#endif
