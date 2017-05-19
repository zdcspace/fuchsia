// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#pragma once

#include "apps/mozart/src/composer/resources/resource.h"
#include "lib/escher/escher/geometry/types.h"

namespace mozart {
namespace composer {

class Shape : public Resource {
 public:
  static const ResourceTypeInfo kTypeInfo;

  /// Returns if the given point lies within its bounds of this shape.
  virtual bool ContainsPoint(const escher::vec2& point) const = 0;

  void Accept(class ResourceVisitor* visitor) override;

 protected:
  Shape(Session* session, const ResourceTypeInfo& type_info);
};

typedef ftl::RefPtr<Shape> ShapePtr;

}  // namespace composer
}  // namespace mozart
