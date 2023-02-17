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

  late int Function(Pointer<Engine> engine, Pointer<Pointer<Utf8>> out)
      source_meta;

  late int Function(Pointer<Pointer<Utf8>> buffer) last_error_message;

  QuelleBindings() {
    quelle = loadDynamicLibrary();

    open_engine_with_path = quelle
        .lookup<NativeFunction<open_engine_with_path_native_t>>(
            "open_engine_with_path")
        .asFunction();
    source_meta = quelle
        .lookup<NativeFunction<source_meta_native_t>>("source_meta")
        .asFunction();
    last_error_message = quelle
        .lookup<NativeFunction<last_error_message_native_t>>(
            "last_error_message")
        .asFunction();
  }
}

QuelleBindings? _cachedBindings;
QuelleBindings get bindings => _cachedBindings ??= QuelleBindings();
