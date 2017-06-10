// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "packet.h"

#include <magenta/assert.h>

#include <algorithm>
#include <utility>

namespace wlan {

Packet::Packet(mxtl::unique_ptr<Buffer> buffer, size_t len)
  : buffer_(std::move(buffer)), len_(len) {
    MX_ASSERT(buffer_.get());
    MX_DEBUG_ASSERT(len <= buffer_->capacity());
}

mx_status_t Packet::CopyFrom(const void* src, size_t len, size_t offset) {
    if (offset + len > buffer_->capacity()) {
        return MX_ERR_BUFFER_TOO_SMALL;
    }
    std::memcpy(buffer_->data() + offset, src, len);
    len_ = std::max(len_, offset + len);
    return MX_OK;
}

mxtl::unique_ptr<Buffer> GetBuffer(size_t len) {
    mxtl::unique_ptr<Buffer> buffer;
    if (len > kSmallBufferSize) {
        buffer = LargeBufferAllocator::New();
    } else {
        buffer = SmallBufferAllocator::New();
        if (buffer == nullptr) {
            // Fall back to the large buffers if we're out of small buffers.
            buffer = LargeBufferAllocator::New();
        }
    }
    return buffer;
}

}  // namespace wlan

// Definition of static slab allocators.
// TODO(tkilbourn): tune how many slabs we are willing to grow up to. Reasonably large limits chosen
// for now.
DECLARE_STATIC_SLAB_ALLOCATOR_STORAGE(::wlan::LargeBufferTraits, 20, true);
DECLARE_STATIC_SLAB_ALLOCATOR_STORAGE(::wlan::SmallBufferTraits, 80, true);
