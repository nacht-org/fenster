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
    return MemLoc(_engine, signedLength).readAsString();
  }

  Meta meta() {
    final map = jsonDecode(metaJson());
    return Meta.parse(map);
  }

  String fetchNovelJson(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());

    try {
      final signedLength =
          bindings.fetch_novel(_engine.unsafe(), urlC.unsafe());
      return MemLoc(_engine, signedLength).readAsString();
    } finally {
      urlC.free();
    }
  }

  Novel fetchNovel(String url) {
    final map = jsonDecode(fetchNovelJson(url));
    return Novel.parse(map);
  }

  String fetchChapterContent(String url) {
    final urlC = Utf8Resource(url.toNativeUtf8());

    try {
      final signedLength =
          bindings.fetch_chapter_content(_engine.unsafe(), urlC.unsafe());
      return MemLoc(_engine, signedLength).readAsString();
    } finally {
      urlC.free();
    }
  }

  bool popularSupported() {
    final result = bindings.popular_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String popularUrl(int page) {
    final length = bindings.popular_url(_engine.unsafe(), page);
    return MemLoc(_engine, length).readAsString();
  }

  String popularJson(int page) {
    final signedLength = bindings.popular(_engine.unsafe(), page);
    return MemLoc(_engine, signedLength).readAsString();
  }

  bool textSearchSupported() {
    final result = bindings.text_search_supported(_engine.unsafe());
    if (result < 0) throw _readError();
    return result > 0;
  }

  String textSearchJson(String query, int page) {
    final queryC = Utf8Resource(query.toNativeUtf8());

    try {
      final signedLength =
          bindings.text_search(_engine.unsafe(), queryC.unsafe(), page);
      return MemLoc(_engine, signedLength).readAsString();
    } finally {
      queryC.free();
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

class MemLoc {
  final EngineResource _engine;
  final int length;

  late final int offset;
  late final Pointer<Uint8> ptr;

  MemLoc(this._engine, this.length) {
    offset = bindings.last_offset();
    ptr = bindings.last_pointer();
  }

  String readAsString() {
    if (length > 0) {
      return _read();
    } else if (length < 0) {
      throw _readResultError(-length);
    } else {
      return '';
    }
  }

  QuelleException _readResultError(length) {
    // check for ffi specific error
    final pointer = bindings.last_result();
    if (pointer.address != nullptr.address) {
      final errorMessage = pointer.toDartString(length: length);
      calloc.free(pointer);
      return QuelleException(errorMessage);
    }

    // read memloc as error
    final errorJson = _read();
    dealloc();
    return QuelleException(errorJson);
  }

  String _read() {
    final value = ptr.asTypedList(length.abs());
    final string = utf8.decode(value);
    dealloc();
    return string;
  }

  void dealloc() {
    final result =
        bindings.memloc_dealloc(_engine.unsafe(), offset, length.abs());
    if (result < 0) {
      throw _readResultError(-result);
    }
  }
}

class QuelleException implements Exception {
  final String message;

  QuelleException(this.message);

  @override
  String toString() => "QuelleException: $message";
}
