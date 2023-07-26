// ignore_for_file: camel_case_types, non_constant_identifier_names

import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'types.dart';

typedef open_engine_with_path_native_t = Int32 Function(
  Pointer<Uint8> path_ptr,
  Uint32 path_len,
  Pointer<NativeFunction<Int32 Function(Pointer<Uint8>, Int32)>> send_request,
  Pointer<NativeFunction<Void Function(Pointer<Uint8>, Int32)>> log_event,
  Pointer<Pointer<Engine>> engine_out,
);

typedef source_meta_native_t = Int32 Function(Pointer<Engine> engine);

typedef fetch_novel_native_t = Int32 Function(
  Pointer<Engine> engine,
  Pointer<Uint8> url_path,
  Uint32 url_len,
);

typedef fetch_chapter_content_native_t = Int32 Function(
  Pointer<Engine> engine,
  Pointer<Uint8> url_path,
  Uint32 url_len,
);

typedef popular_suppported_native_t = Int32 Function(
  Pointer<Engine> engine,
);

typedef popular_url_native_t = Int32 Function(
  Pointer<Engine> engine,
  Int32 page,
);

typedef popular_native_t = Int32 Function(
  Pointer<Engine> engine,
  Int32 page,
);

typedef text_search_native_t = Int32 Function(
  Pointer<Engine> engine,
  Pointer<Uint8> query_ptr,
  Uint32 query_len,
  Int32 page,
);

typedef text_search_supported_native_t = Int32 Function(
  Pointer<Engine> engine,
);

typedef last_result_native_t = Pointer<Utf8> Function();

typedef last_pointer_native_t = Pointer<Uint8> Function();

typedef set_last_pointer_native_t = Void Function(Pointer<Uint8> value);

typedef last_offset_native_t = Int32 Function();

typedef set_last_offset_native_t = Void Function(Int32 value);

typedef memloc_dealloc_native_t = Int32 Function(
    Pointer<Engine> engine, Int32 ptr, Int32 len);
