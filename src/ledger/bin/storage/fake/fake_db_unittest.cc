// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "src/ledger/bin/storage/fake/fake_db.h"

#include <memory>

#include "src/ledger/bin/storage/public/db_unittest.h"

namespace storage {
namespace fake {
namespace {

std::unique_ptr<Db> GetDb(scoped_tmpfs::ScopedTmpFS* tmpfs, async_dispatcher_t* dispatcher) {
  return std::make_unique<FakeDb>(dispatcher);
}

INSTANTIATE_TEST_SUITE_P(FakeDbTest, DbTest, testing::Values(&GetDb));
}  // namespace
}  // namespace fake
}  // namespace storage
