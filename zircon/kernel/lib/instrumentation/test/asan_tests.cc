// Copyright 2020 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#include <lib/instrumentation/asan.h>
#include <lib/unittest/unittest.h>
#include <platform.h>
#include <zircon/types.h>

#include <new>

#include <fbl/alloc_checker.h>
#include <kernel/thread.h>
#include <kernel/wait.h>
#include <ktl/array.h>
#include <ktl/limits.h>
#include <ktl/unique_ptr.h>
#include <vm/physmap.h>
#include <vm/pmm.h>

#if __has_feature(address_sanitizer)
namespace {

constexpr size_t kAsanShift = 3;
constexpr size_t kAsanGranularity = (1 << kAsanShift);

// Makes sure that a region returned by malloc are unpoisoned.
bool kasan_test_malloc_poisons() {
  BEGIN_TEST;
  constexpr size_t sizes[] = {1, 10, 32, 1023, 1024};

  for (auto size : sizes) {
    void* m = malloc(size);
    ASSERT_NONNULL(m);
    EXPECT_EQ(0UL, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(m), size));
    free(m);
  }
  END_TEST;
}

// Makes sure that a region recently freed is poisoned.
bool kasan_test_free_poisons() {
  BEGIN_TEST;
  constexpr size_t sizes[] = {1, 10, 32, 1023, 1024};

  for (auto size : sizes) {
    void* m = malloc(size);
    ASSERT_NONNULL(m);
    free(m);
    EXPECT_TRUE(asan_entire_region_is_poisoned(reinterpret_cast<uintptr_t>(m), size));
  }
  END_TEST;
}

// Makes sure that the surrounding parts of a buffer are poisoned.
bool kasan_test_detects_buffer_overflows() {
  BEGIN_TEST;
  constexpr size_t sizes[] = {1, 2, 3, 4, 5, 6, 7, 10, 32, 1023, 1024};

  for (auto size : sizes) {
    void* m = malloc(size);
    ASSERT_NONNULL(m);
    uintptr_t base = reinterpret_cast<uintptr_t>(m);
    EXPECT_TRUE(asan_address_is_poisoned(base + size));
    EXPECT_TRUE(asan_address_is_poisoned(base - 1));
    free(m);
  }
  END_TEST;
}

// Makes sure that a regions from the heap can be poisoned.
bool kasan_test_poison_heap() {
  BEGIN_TEST;

  constexpr size_t sizes[] = {1, 2, 3, 5, 7, 8, 9, 11, 15, 16, 17, 19};

  constexpr size_t kBufSz = 1024;
  fbl::AllocChecker ac;
  auto buf = ktl::unique_ptr<uint8_t[]>(new (&ac) uint8_t[kBufSz]);
  ASSERT(ac.check());
  ASSERT_EQ(0UL, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));
  for (size_t size : sizes) {
    size_t poisoned_size = ROUNDDOWN(size, 1 << kAsanShift);
    asan_poison_shadow(reinterpret_cast<uintptr_t>(buf.get()), size, kAsanHeapLeftRedzoneMagic);
    EXPECT_TRUE(
        asan_entire_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), poisoned_size));
    asan_unpoison_shadow(reinterpret_cast<uintptr_t>(buf.get()), kBufSz);
    EXPECT_EQ(0UL, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));
  }

  END_TEST;
}

// Make sure poison checks works in partially poisoned regions.
bool kasan_test_poison_heap_partial() {
  BEGIN_TEST;

  constexpr size_t kBufSz = 128;
  fbl::AllocChecker ac;
  auto buf = ktl::unique_ptr<uint8_t[]>(new (&ac) uint8_t[kBufSz]);
  ASSERT(ac.check());
  ASSERT_EQ(0UL, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));

  // Leave the first and last two granules unpoisoned.
  const uintptr_t poison_start = reinterpret_cast<uintptr_t>(buf.get()) + (2 << kAsanShift);
  const size_t poison_size = kBufSz - (4 << kAsanShift);

  asan_poison_shadow(poison_start, poison_size, kAsanHeapLeftRedzoneMagic);
  EXPECT_EQ(poison_start, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));

  asan_unpoison_shadow(poison_start, poison_size);
  ASSERT_EQ(0UL, asan_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));

  END_TEST;
}

bool kasan_test_poison_unaligned_offsets() {
  BEGIN_TEST;

  constexpr size_t kBufSz = 128;
  fbl::AllocChecker ac;
  auto buf = ktl::unique_ptr<uint8_t[]>(new (&ac) uint8_t[kBufSz]);
  ASSERT(ac.check());
  uintptr_t bufptr = reinterpret_cast<uintptr_t>(buf.get());
  ASSERT_EQ(0UL, asan_region_is_poisoned(bufptr, kBufSz));

  // Poison from byte 4 onwards.
  size_t poison_skip = 3;
  asan_poison_shadow(bufptr + poison_skip, kBufSz - poison_skip, kAsanHeapLeftRedzoneMagic);
  EXPECT_EQ(bufptr + poison_skip, asan_region_is_poisoned(bufptr, kBufSz));
  EXPECT_TRUE(asan_entire_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()) + poison_skip,
                                             kBufSz - poison_skip));

  // Unpoison the last chunk
  size_t unpoison_start = 2 * kAsanGranularity - 1;
  asan_unpoison_shadow(bufptr + unpoison_start, kBufSz - unpoison_start);
  EXPECT_EQ(0UL, asan_region_is_poisoned(bufptr + unpoison_start, kBufSz - unpoison_start));
  // It didn't unpoison the first asan granule.
  EXPECT_TRUE(asan_entire_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()) + poison_skip,
                                             kAsanGranularity - poison_skip));

  // Poisoning the third byte onwards should increase the left poison size.
  asan_poison_shadow(bufptr + poison_skip - 1, kBufSz - poison_skip + 1, kAsanHeapLeftRedzoneMagic);
  EXPECT_EQ(bufptr + poison_skip - 1, asan_region_is_poisoned(bufptr, kBufSz));
  EXPECT_TRUE(asan_entire_region_is_poisoned(
      reinterpret_cast<uintptr_t>(buf.get()) + poison_skip - 1, kBufSz - poison_skip + 1));

  // Unpoison the fourth byte should unpoison bytes 0, 1, 2 and 3.
  asan_unpoison_shadow(bufptr + poison_skip, 1);
  EXPECT_EQ(bufptr + poison_skip + 1, asan_region_is_poisoned(bufptr, kBufSz));
  EXPECT_TRUE(asan_entire_region_is_poisoned(
      reinterpret_cast<uintptr_t>(buf.get()) + poison_skip + 1, kBufSz - poison_skip));

  END_TEST;
}

// Make sure poisoning less than an entire granule works.
bool kasan_test_poison_small() {
  BEGIN_TEST;

  constexpr size_t kBufSz = 128;
  fbl::AllocChecker ac;
  auto buf = ktl::unique_ptr<uint8_t[]>(new (&ac) uint8_t[kBufSz]);
  ASSERT(ac.check());
  uintptr_t bufptr = reinterpret_cast<uintptr_t>(buf.get());
  ASSERT_EQ(0UL, asan_region_is_poisoned(bufptr, kBufSz));

  // If we try to poison the first kAsanGranularity - 1 bytes into an unpoisoned
  // region, it shouldn't do anything.
  asan_poison_shadow(bufptr, kAsanGranularity - 1, kAsanHeapLeftRedzoneMagic);
  ASSERT_EQ(0UL, asan_region_is_poisoned(bufptr, kBufSz));

  // Poison from byte 3 onwards.
  size_t poison_skip = 3;
  asan_poison_shadow(bufptr + poison_skip, kBufSz - poison_skip, kAsanHeapLeftRedzoneMagic);
  ASSERT_EQ(bufptr + poison_skip, asan_region_is_poisoned(bufptr, kBufSz));

  // If we poison the first 2 bytes, nothing should happen.
  asan_poison_shadow(bufptr, poison_skip - 1, kAsanHeapLeftRedzoneMagic);
  EXPECT_EQ(bufptr + poison_skip, asan_region_is_poisoned(bufptr, kBufSz));

  // Poisoning the third byte should increase the poisoned region.
  asan_poison_shadow(bufptr + poison_skip - 1, 1, kAsanHeapLeftRedzoneMagic);
  EXPECT_EQ(bufptr + poison_skip - 1, asan_region_is_poisoned(bufptr, kBufSz));

  // Poisoning the from the start should make the whole range poisoned.
  asan_poison_shadow(bufptr, poison_skip, kAsanHeapLeftRedzoneMagic);
  EXPECT_TRUE(asan_entire_region_is_poisoned(reinterpret_cast<uintptr_t>(buf.get()), kBufSz));

  asan_unpoison_shadow(bufptr, kBufSz);
  ASSERT_EQ(0UL, asan_region_is_poisoned(bufptr, kBufSz));

  END_TEST;
}

bool kasan_test_quarantine() {
  BEGIN_TEST;
  static constexpr uintptr_t kFirstPtr = 0xA1FA1FA;
  fbl::AllocChecker ac;
  auto test_quarantine = ktl::make_unique<asan::Quarantine>(&ac);
  ASSERT(ac.check());

  for (size_t i = 0; i < asan::Quarantine::kQuarantineElements; i++) {
    void* const fake_pointer = reinterpret_cast<void*>(kFirstPtr + i);
    void* const retrieved = test_quarantine->push(fake_pointer);
    EXPECT_EQ(retrieved, nullptr);
  }
  for (size_t i = 0; i < asan::Quarantine::kQuarantineElements; i++) {
    void* const retrieved = test_quarantine->push(nullptr);
    EXPECT_EQ(retrieved, reinterpret_cast<void*>(kFirstPtr + i));
  }
  EXPECT_EQ(nullptr, test_quarantine->push(nullptr));

  END_TEST;
}

}  // namespace

UNITTEST_START_TESTCASE(kasan_tests)
UNITTEST("small_poison", kasan_test_poison_small)
UNITTEST("unaligned_poison", kasan_test_poison_unaligned_offsets)
UNITTEST("malloc_unpoisons", kasan_test_malloc_poisons)
// TODO(fxb/52129): Test is flaky. Fix and re-enable.
// UNITTEST("free_poisons", kasan_test_free_poisons)
UNITTEST("detects_buffer_overflows", kasan_test_detects_buffer_overflows)
UNITTEST("test_poisoning_heap", kasan_test_poison_heap)
UNITTEST("test_poisoning_heap_partial", kasan_test_poison_heap_partial)
UNITTEST("test_quarantine", kasan_test_quarantine)
UNITTEST_END_TESTCASE(kasan_tests, "kasan", "Kernel Address Sanitizer Tests")

#endif  // _has_feature(address_sanitizer)
