// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef SRC_GRAPHICS_DRIVERS_MSD_VSL_GC_INCLUDE_MAGMA_VENDOR_QUERIES_H_
#define SRC_GRAPHICS_DRIVERS_MSD_VSL_GC_INCLUDE_MAGMA_VENDOR_QUERIES_H_

#include "magma_common_defs.h"

#define MAGMA_VENDOR_ID_VSL 0x10001  // VK_VENDOR_ID_VIV

enum MsdVslVendorQuery {
  kMsdVslVendorQueryChipIdentity = MAGMA_QUERY_VENDOR_PARAM_0,
  kMsdVslVendorQueryChipOption = MAGMA_QUERY_VENDOR_PARAM_0 + 1,
  kMsdVslVendorQueryClientGpuAddrRange = MAGMA_QUERY_VENDOR_PARAM_0 + 2,
};

#endif  // SRC_GRAPHICS_DRIVERS_MSD_VSL_GC_INCLUDE_MAGMA_VENDOR_QUERIES_H_
