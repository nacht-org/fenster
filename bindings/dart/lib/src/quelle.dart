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
    final signedLength = bindings.source_meta(_engine.unsafe());
    return _readMemLoc(signedLength);
  }

  Meta meta() {
    final map = jsonDecode(metaJson());
    return Meta.parse(map);
  }

  String fetchNovelJson(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());
    String json;

    try {
      final signedLength =
          bindings.fetch_novel(_engine.unsafe(), urlC.unsafe());
      json = _readStringResult(signedLength);
    } finally {
      urlC.free();
    }

    return json;
  }

  Novel fetchNovel(String url) {
    final map = jsonDecode(fetchNovelJson(url));
    return Novel.parse(map);
  }

  String fetchChapterContent(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());
    String content;

    try {
      final signedLength =
          bindings.fetch_chapter_content(_engine.unsafe(), urlC.unsafe());
      content = _readStringResult(signedLength);
    } finally {
      urlC.free();
    }

    return content;
  }

  bool popularSupported() {
    final result = bindings.popular_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String popularJson(int page) {
    final signedLength = bindings.popular(_engine.unsafe(), page);
    return _readStringResult(signedLength);
  }

  bool textSearchSupported() {
    final result = bindings.text_search_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String textSearchJson(String query, int page) {
    final queryC = Utf8Resource(query.toNativeUtf8());
    String content;

    try {
      final signedLength =
          bindings.text_search(_engine.unsafe(), queryC.unsafe(), page);
      content = _readStringResult(signedLength);
    } finally {
      queryC.free();
    }

    return content;
  }

  String _readStringResult(int signedLength) {
    if (signedLength > 0) {
      final pointer = bindings.last_result();
      final value = pointer.toDartString(length: signedLength);
      calloc.free(pointer);
      return value;
    } else if (signedLength < 0) {
      final pointer = bindings.last_result();
      final errorMessage = pointer.toDartString(length: -signedLength);
      calloc.free(pointer);
      throw QuelleException(errorMessage);
    } else {
      return '';
    }
  }

  String _readMemLoc(int signedLength) {
    if (signedLength > 0) {
      final pointer = bindings.last_pointer();
      final offset = bindings.last_offset();

      final value = pointer.asTypedList(signedLength);
      final string = utf8.decode(value);
      dealloc(offset, signedLength);
      return string;
    } else if (signedLength < 0) {
      throw _readResultError(-signedLength);
    } else {
      return '';
    }
  }

  QuelleException _readError() {
    Pointer<Pointer<Utf8>> buffer = calloc();
    bindings.last_error_message(buffer);
    final errorMessage = buffer.value.toDartString();
    print(errorMessage);
    calloc.free(buffer.value);
    calloc.free(buffer);
    return QuelleException(errorMessage);
  }

  QuelleException _readResultError(length) {
    final pointer = bindings.last_result();
    final errorMessage = pointer.toDartString(length: length);
    calloc.free(pointer);
    throw QuelleException(errorMessage);
  }

  void dealloc(int offset, int length) {
    final result = bindings.memloc_dealloc(_engine.unsafe(), offset, length);
    if (result < 0) {
      throw _readResultError(-result);
    }
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
