/**
 * @file pico/stdlib.h
 * @brief Host-side stub of the Pico SDK header, used for syntax checking only.
 *
 * This is NOT used when building the real firmware; it only lets clang
 * -fsyntax-only validate our own sources without the Pico SDK installed.
 */
#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

typedef struct {
  uint64_t ticks;
} absolute_time_t;

void stdio_init_all(void);
absolute_time_t get_absolute_time(void);
absolute_time_t make_timeout_time_ms(uint32_t ms);
absolute_time_t delayed_by_ms(absolute_time_t base, uint32_t ms);
int64_t absolute_time_diff_us(absolute_time_t from, absolute_time_t to);
