import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings/bindings.dart';
import 'bindings/types.dart' as types;
import 'models/models.dart';

class Quelle {
  final String path;

  late EngineResource _engine;

  Quelle(this.path) {
    Pointer<Pointer<types.Engine>> engineOut = calloc();
    final pathC = Utf8Resource(path.toNativeUtf8());

    try {
      final result = bindings.open_engine_with_path(pathC.unsafe(), engineOut);
      if (result != 0) throw _readError();
      _engine = EngineResource(engineOut.value);
    } finally {
      calloc.free(engineOut);
      pathC.free();
    }
  }

  String metaJson() {
    Pointer<Pointer<Utf8>> buffer = calloc();
    String json;

    try {
      final result = bindings.source_meta(_engine.unsafe(), buffer);
      if (result != 0) throw _readError();
      json = buffer.value.toDartString();
    } finally {
      calloc.free(buffer.value);
      calloc.free(buffer);
    }

    return json;
  }

  Meta meta() {
    final map = jsonDecode(metaJson());
    return Meta.parse(map);
  }

  String fetchNovelJson(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());
    Pointer<Pointer<Utf8>> buffer = calloc();
    String json;

    try {
      final result =
          bindings.fetch_novel(_engine.unsafe(), urlC.unsafe(), buffer);
      if (result != 0) throw _readError();
      json = buffer.value.toDartString();
    } finally {
      urlC.free();
      calloc.free(buffer.value);
      calloc.free(buffer);
    }

    return json;
  }

  Novel fetchNovel(String url) {
    final map = jsonDecode(fetchNovelJson(url));
    return Novel.parse(map);
  }

  String fetchChapterContent(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());
    Pointer<Pointer<Utf8>> buffer = calloc();
    String content;

    try {
      final result = bindings.fetch_chapter_content(
          _engine.unsafe(), urlC.unsafe(), buffer);
      if (result != 0) throw _readError();
      content = buffer.value.toDartString();
    } finally {
      urlC.free();
      calloc.free(buffer.value);
      calloc.free(buffer);
    }

    return content;
  }

  bool popularSupported() {
    final result = bindings.popular_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String popularJson(int page) {
    Pointer<Pointer<Utf8>> buffer = calloc();
    String content;

    try {
      final result = bindings.popular(_engine.unsafe(), page, buffer);
      if (result != 0) throw _readError();
      content = buffer.value.toDartString();
    } finally {
      calloc.free(buffer.value);
      calloc.free(buffer);
    }

    return content;
  }

  bool textSearchSupported() {
    final result = bindings.text_search_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String textSearchJson(String query, int page) {
    final queryC = Utf8Resource(query.toNativeUtf8());
    Pointer<Pointer<Utf8>> buffer = calloc();
    String content;

    try {
      final result =
          bindings.text_search(_engine.unsafe(), queryC.unsafe(), page, buffer);
      if (result != 0) throw _readError();
      content = buffer.value.toDartString();
    } finally {
      queryC.free();
      calloc.free(buffer.value);
      calloc.free(buffer);
    }

    return content;
  }

  QuelleException _readError() {
    Pointer<Pointer<Utf8>> buffer = calloc();
    bindings.last_error_message(buffer);
    final errorMessage = buffer.value.toDartString();
    calloc.free(buffer.value);
    calloc.free(buffer);
    return QuelleException(errorMessage);
  }
}

class EngineResource implements Finalizable {
  static final NativeFinalizer _finalizer = NativeFinalizer(posixFree);

  late final Pointer<types.Engine> _engine;

  EngineResource(this._engine) {
    _finalizer.attach(this, _engine.cast(), detach: this);
  }

  void free() {
    _finalizer.detach(this);
    calloc.free(_engine);
  }

  Pointer<types.Engine> unsafe() => _engine;
}

class Utf8Resource implements Finalizable {
  static final NativeFinalizer _finalizer = NativeFinalizer(posixFree);

  /// [_cString] must never escape [Utf8Resource], otherwise the
  /// [_finalizer] will run prematurely.
  final Pointer<Utf8> _cString;

  Utf8Resource(this._cString) {
    _finalizer.attach(this, _cString.cast(), detach: this);
  }

  void free() {
    _finalizer.detach(this);
    calloc.free(_cString);
  }

  /// Ensure this [Utf8Resource] stays in scope longer than the inner resource.
  Pointer<Utf8> unsafe() => _cString;
}

final DynamicLibrary stdlib = DynamicLibrary.process();
final posixFree = stdlib.lookup<NativeFunction<Void Function(Pointer)>>("free");

class QuelleException implements Exception {
  final String message;

  QuelleException(this.message);

  @override
  String toString() => "QuelleException: $message";
}
