// Copyright 2016 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#include "lib/cmdline.h"

#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <zircon/assert.h>

Cmdline gCmdline;

void Cmdline::Append(const char* str) {
  if (str == nullptr || *str == 0) {
    return;
  }

  bool found_equal = false;
  for (;;) {
    unsigned c = *str++;
    if (c == 0) {
      // Finish an in-progress argument.
      if (length_ > 0 && data_[length_ - 1] != 0) {
        if (!found_equal) {
          AddOrAbort('=');
        }
        // Terminate the string.
        AddOrAbort(0);
      }
      break;
    }

    if (c == '=') {
      found_equal = true;
    } else if ((c < ' ') || (c > 127)) {
      // It's a special character of some kind.
      if ((c == '\n') || (c == '\r') || (c == '\t')) {
        c = ' ';
      } else {
        c = '.';
      }
    }

    if (c == ' ') {
      // Spaces become \0's, but do not double up.
      if (length_ == 0 || (data_[length_ - 1] == 0)) {
        // No need to add another terminator, so loop back to the start.
        continue;
      }

      if (!found_equal) {
        AddOrAbort('=');
      } else {
        found_equal = false;
      }
      // Add the terminator.
      AddOrAbort(0);
      continue;
    }

    AddOrAbort(static_cast<char>(c));
  }
}

const char* Cmdline::GetString(const char* key) const {
  if (!key) {
    return data_;
  }

  const size_t sz = strlen(key);
  if (sz == 0) {
    return nullptr;
  }

  if (length_ == 0) {
    return nullptr;
  }

  const char* ptr = data_;
  for (;;) {
    if (!strncmp(ptr, key, sz) && (ptr[sz] == '=' || ptr[sz] == '\0')) {
      break;
    }
    ptr = strchr(ptr, 0) + 1;
    if (*ptr == 0) {
      return nullptr;
    }
  }
  ptr += sz;
  if (*ptr == '=') {
    ptr++;
  }
  return ptr;
}

bool Cmdline::GetBool(const char* key, bool default_value) const {
  const char* value = GetString(key);
  if (value == nullptr) {
    return default_value;
  }
  if ((strcmp(value, "0") == 0) || (strcmp(value, "false") == 0) || (strcmp(value, "off") == 0)) {
    return false;
  }
  return true;
}

uint32_t Cmdline::GetUInt32(const char* key, uint32_t default_value) const {
  const char* value_str = GetString(key);
  if (value_str == nullptr || *value_str == '\0') {
    return default_value;
  }

  char* end;
  auto res = strtol(value_str, &end, 0);
  uint32_t value = (res < 0) ? default_value : static_cast<uint32_t>(res);
  if (*end != '\0') {
    return default_value;
  }
  return value;
}

uint64_t Cmdline::GetUInt64(const char* key, uint64_t default_value) const {
  const char* value_str = GetString(key);
  if (value_str == nullptr || *value_str == '\0') {
    return default_value;
  }

  char* end;
  long long value = strtoll(value_str, &end, 0);
  if (*end != '\0') {
    return default_value;
  }
  return value;
}

size_t Cmdline::size() const {
  return length_ + 1;
}


void Cmdline::AddOrAbort(char c) {
  if (length_ < kCmdlineMax - 1) {
    data_[length_++] = c;
  } else {
    ZX_PANIC("cmdline overflow");
  }
}
