// ignore_for_file: camel_case_types, non_constant_identifier_names

import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'types.dart';

typedef open_engine_with_path_native_t = Int32 Function(
    Pointer<Utf8> path, Pointer<Pointer<Engine>> engine_out);

typedef source_meta_native_t = Int32 Function(
    Pointer<Engine> engine, Pointer<Pointer<Utf8>> buffer);

typedef last_error_message_native_t = Int32 Function(
    Pointer<Pointer<Utf8>> buffer);

typedef fetch_novel_native_t = Int32 Function(
  Pointer<Engine> engine,
  Pointer<Utf8> url,
  Pointer<Pointer<Utf8>> buffer,
);

typedef fetch_chapter_content_native_t = Int32 Function(
  Pointer<Engine> engine,
  Pointer<Utf8> url,
  Pointer<Pointer<Utf8>> buffer,
);

typedef popular_native_t = Int32 Function(
  Pointer<Engine> engine,
  Int32 page,
  Pointer<Pointer<Utf8>> buffer,
);
