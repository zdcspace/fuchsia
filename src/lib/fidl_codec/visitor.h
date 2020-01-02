// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef SRC_LIB_FIDL_CODEC_VISITOR_H_
#define SRC_LIB_FIDL_CODEC_VISITOR_H_

#include "src/lib/fidl_codec/wire_object.h"

namespace fidl_codec {

// Superclass for implementing visitors for Values. Note that the whole class is protected. To use a
// visitor, use the Visit method on the Value object you want to visit.
class Visitor {
 protected:
  virtual void VisitValue(const Value* node, const Type* for_type) {}
  virtual void VisitInvalidValue(const InvalidValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitNullValue(const NullValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitRawValue(const RawValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitStructValue(const StructValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitStringValue(const StringValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitBoolValue(const BoolValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitTableValue(const TableValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitUnionValue(const UnionValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitVectorValue(const VectorValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitEnumValue(const EnumValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitBitsValue(const BitsValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitHandleValue(const HandleValue* node, const Type* for_type) {
    VisitValue(node, for_type);
  }

  virtual void VisitU8Value(const NumericValue<uint8_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitU16Value(const NumericValue<uint16_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitU32Value(const NumericValue<uint32_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitU64Value(const NumericValue<uint64_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitI8Value(const NumericValue<int8_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitI16Value(const NumericValue<int16_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitI32Value(const NumericValue<int32_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitI64Value(const NumericValue<int64_t>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitF32Value(const NumericValue<float>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }
  virtual void VisitF64Value(const NumericValue<double>* node, const Type* for_type) {
    VisitValue(node, for_type);
  }

  friend class Value;
  friend class InvalidValue;
  friend class NullValue;
  friend class RawValue;
  template <typename T>
  friend class NumericValue;
  friend class StringValue;
  friend class BoolValue;
  friend class StructValue;
  friend class TableValue;
  friend class UnionValue;
  friend class VectorValue;
  friend class EnumValue;
  friend class BitsValue;
  friend class HandleValue;
};

}  // namespace fidl_codec

#endif  // SRC_LIB_FIDL_CODEC_VISITOR_H_
