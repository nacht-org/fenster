// ignore_for_file: non_constant_identifier_names

import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'signatures.dart';
import 'types.dart';
import '../locator.dart';

class QuelleBindings {
  late DynamicLibrary quelle;

  late int Function(Pointer<Utf8> path, Pointer<Pointer<Engine>> engine_out)
      open_engine_with_path;

  late int Function(Pointer<Engine> engine, int ptr, int len) memloc_dealloc;

  late int Function(Pointer<Engine> engine) source_meta;

  late int Function(Pointer<Engine> engine, Pointer<Utf8> url) fetch_novel;

  late int Function(Pointer<Engine> engine, Pointer<Utf8> url)
      fetch_chapter_content;

  late int Function(Pointer<Engine> engine) popular_supported;

  late int Function(Pointer<Engine> engine, int page) popular;

  late int Function(
    Pointer<Engine> engine,
    Pointer<Utf8> query,
    int page,
  ) text_search;

  late int Function(Pointer<Engine> engine) text_search_supported;

  late int Function(Pointer<Pointer<Utf8>> buffer) last_error_message;

  late Pointer<Utf8> Function() last_result;

  late Pointer<Uint8> Function() last_pointer;

  late int Function() last_offset;

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
    popular =
        quelle.lookup<NativeFunction<popular_native_t>>("popular").asFunction();
    text_search = quelle
        .lookup<NativeFunction<text_search_native_t>>("text_search")
        .asFunction();
    text_search_supported = quelle
        .lookup<NativeFunction<text_search_supported_native_t>>(
            "text_search_supported")
        .asFunction();
    last_error_message = quelle
        .lookup<NativeFunction<last_error_message_native_t>>(
            "last_error_message")
        .asFunction();
    last_result = quelle
        .lookup<NativeFunction<last_result_native_t>>("last_result")
        .asFunction();
    last_pointer = quelle
        .lookup<NativeFunction<last_pointer_native_t>>('last_pointer')
        .asFunction();
    last_offset = quelle
        .lookup<NativeFunction<last_offset_native_t>>('last_offset')
        .asFunction();
  }
}

QuelleBindings? _cachedBindings;
QuelleBindings get bindings => _cachedBindings ??= QuelleBindings();
