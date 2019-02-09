// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include <fuchsia/device/c/fidl.h>
#include <lib/fdio/util.h>
#include <zircon/syscalls.h>
#include <zircon/status.h>
#include <zircon/types.h>

#include <stdio.h>

int main(int argc, char* argv[]) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <device path>\n", argv[0]);
        return -1;
    }

    zx_handle_t local, remote;
    zx_status_t status = zx_channel_create(0, &local, &remote);
    if (status != ZX_OK) {
        fprintf(stderr, "could not create channel\n");
        return -1;
    }
    status = fdio_service_connect(argv[1], remote);
    if (status != ZX_OK) {
        fprintf(stderr, "could not open %s: %s\n", argv[1], zx_status_get_string(status));
        return -1;
    }

    char path[1025];
    zx_status_t call_status;
    size_t actual_len;
    status = fuchsia_device_ControllerGetTopologicalPath(local, &call_status, path,
                                                         sizeof(path) - 1, &actual_len);
    if (status == ZX_OK) {
        status = call_status;
    }
    if (status != ZX_OK) {
        fprintf(stderr, "could not get topological path for %s: %s\n",
                argv[1], zx_status_get_string(status));
        return -1;
    }
    path[actual_len] = 0;

    printf("topological path for %s: %s\n", argv[1], path);
    return 0;
}
