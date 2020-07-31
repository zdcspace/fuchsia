// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// @dart = 2.8

/// This library defines the query report export format.
///
/// ## Overview
///
/// This abstract syntax tree (AST) is designed for printing query reports.
/// Its generic hierarchical nature could render into a variety of formats:
/// terminal, JSON, HTML, etc.
library ast;

import 'dart:core';

import '../codesize.dart';

/// Anything that could be used as the content of a row.
/// A title could be plain text (see `StyledString.plain`), some styled text
/// (see `StyledString`), or some structured object (see `SizeRecord`).
abstract class Title {}

/// Helper type to erase the generic argument from `Node<T>`.
abstract class AnyNode {
  Title get title;
  List<AnyNode> get children;
}

/// The building block of the hierarchial AST.
/// Each node has a title, and optionally some children.
class Node<T extends Title> extends AnyNode {
  @override
  final T title;

  @override
  final List<AnyNode> children;

  Node({this.title, this.children});

  /// Creates a simple node with plain text as title, and no children.
  // Use static methods due to https://github.com/dart-lang/sdk/issues/26391
  // ignore: prefer_constructors_over_static_methods
  static Node<StyledString> plain(String plain) =>
      Node(title: StyledString.plain(plain), children: []);
}

// Styled string ---------------------------------------------------------------

abstract class StringPiece {}

class StyledString extends Title implements StringPiece {
  final List<StringPiece> details;

  StyledString(this.details);

  StyledString.plain(String plain) : details = ([Plain(plain)]);
}

/// Plain text.
class Plain implements StringPiece {
  final String text;

  Plain(this.text);
}

enum Color { white, green, gray }

/// Instructs the renderer to color the contained `StringPiece` a certain color.
class AddColor extends StyledString {
  final Color color;

  AddColor(this.color, StringPiece piece) : super([piece]);

  AddColor.white(StringPiece piece)
      : color = Color.white,
        super([piece]);
  AddColor.green(StringPiece piece)
      : color = Color.green,
        super([piece]);
  AddColor.gray(StringPiece piece)
      : color = Color.gray,
        super([piece]);
}

// Custom title components -----------------------------------------------------

class SizeRecord extends Title {
  final StyledString name;
  final Tally tally;

  SizeRecord({this.name, this.tally});
}

class UniqueSymbolSizeRecord extends SizeRecord {
  final List<StyledString> categories;
  final List<StyledString> rustCrates;

  UniqueSymbolSizeRecord(
      {StyledString name, Tally tally, this.categories, this.rustCrates})
      : super(name: name, tally: tally);
}
