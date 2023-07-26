// ignore_for_file: non_constant_identifier_names

import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'signatures.dart';
import 'types.dart';
import '../locator.dart';

class QuelleBindings {
  late DynamicLibrary quelle;

  late int Function(
    Pointer<Uint8> path_ptr,
    int path_len,
    Pointer<NativeFunction<Int32 Function(Pointer<Uint8>, Int32)>> send_request,
    Pointer<NativeFunction<Void Function(Pointer<Uint8>, Int32)>> log_event,
    Pointer<Pointer<Engine>> engine_out,
  ) open_engine_with_path;

  late int Function(Pointer<Engine> engine, int ptr, int len) memloc_dealloc;

  late int Function(Pointer<Engine> engine) source_meta;

  late int Function(Pointer<Engine> engine, Pointer<Uint8> url_ptr, int url_len)
      fetch_novel;

  late int Function(Pointer<Engine> engine, Pointer<Uint8> url_ptr, int url_len)
      fetch_chapter_content;

  late int Function(Pointer<Engine> engine) popular_supported;

  late int Function(Pointer<Engine> engine, int page) popular_url;

  late int Function(Pointer<Engine> engine, int page) popular;

  late int Function(
    Pointer<Engine> engine,
    Pointer<Uint8> query_ptr,
    int query_len,
    int page,
  ) text_search;

  late int Function(Pointer<Engine> engine) text_search_supported;

  late Pointer<Utf8> Function() last_result;

  late Pointer<Uint8> Function() last_pointer;

  late void Function(Pointer<Uint8> value) set_last_pointer;

  late int Function() last_offset;

  late void Function(int value) set_last_offset;

  QuelleBindings() {
    quelle = loadDynamicLibrary();

    open_engine_with_path = quelle
        .lookup<NativeFunction<open_engine_with_path_native_t>>(
            "open_engine_with_path")
        .asFunction();
    memloc_dealloc = quelle
        .lookup<NativeFunction<memloc_dealloc_native_t>>('memloc_dealloc')
        .asFunction();
    source_meta = quelle
        .lookup<NativeFunction<source_meta_native_t>>("source_meta")
        .asFunction();
    fetch_novel = quelle
        .lookup<NativeFunction<fetch_novel_native_t>>("fetch_novel")
        .asFunction();
    fetch_chapter_content = quelle
        .lookup<NativeFunction<fetch_chapter_content_native_t>>(
            "fetch_chapter_content")
        .asFunction();
    popular_supported = quelle
        .lookup<NativeFunction<popular_suppported_native_t>>(
            "popular_supported")
        .asFunction();
    popular_url = quelle
        .lookup<NativeFunction<popular_url_native_t>>('popular_url')
        .asFunction();
    popular =
        quelle.lookup<NativeFunction<popular_native_t>>("popular").asFunction();
    text_search = quelle
        .lookup<NativeFunction<text_search_native_t>>("text_search")
        .asFunction();
    text_search_supported = quelle
        .lookup<NativeFunction<text_search_supported_native_t>>(
            "text_search_supported")
        .asFunction();
    last_result = quelle
        .lookup<NativeFunction<last_result_native_t>>("last_result")
        .asFunction();
    last_pointer = quelle
        .lookup<NativeFunction<last_pointer_native_t>>('last_pointer')
        .asFunction();
    set_last_pointer = quelle
        .lookup<NativeFunction<set_last_pointer_native_t>>('set_last_pointer')
        .asFunction();
    last_offset = quelle
        .lookup<NativeFunction<last_offset_native_t>>('last_offset')
        .asFunction();
    set_last_offset = quelle
        .lookup<NativeFunction<set_last_offset_native_t>>('set_last_offset')
        .asFunction();
  }
}

QuelleBindings? _cachedBindings;
QuelleBindings get bindings => _cachedBindings ??= QuelleBindings();
