import 'dart:convert';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:ffi/ffi.dart';

import 'bindings/bindings.dart';
import 'bindings/types.dart' as types;
import 'models/models.dart';

extension on String {
  Pointer<Utf8> toNativeUtf8Unterminated({Allocator allocator = malloc}) {
    final units = utf8.encode(this);
    final Pointer<Uint8> result = allocator<Uint8>(units.length);
    final Uint8List nativeString = result.asTypedList(units.length);
    nativeString.setAll(0, units);
    return result.cast();
  }
}

/// FIXME: waiting for language support
///
/// this does not currently work due to lack of support for async callbacks.
/// waiting on https://github.com/dart-lang/sdk/issues/37022.
int sendRequest(Pointer<Uint8> pointer, int length) {
  final bytes = pointer.asTypedList(length);
  final map = jsonDecode(utf8.decode(bytes));
  print(map);

  Future.delayed(const Duration(milliseconds: 100))
      .whenComplete(() => print('complete'));

  // Dio()
  //     .request(
  //   map['url'],
  //   data: map['data'],
  //   queryParameters: map['params'],
  //   options: Options(
  //     method: map['method'],
  //     headers: map['headers'],
  //     responseType: ResponseType.bytes,
  //   ),
  // )
  //     .then((value) {
  //   print(value);
  //   writeString(jsonEncode({
  //     'status': value.statusCode,
  //     'body': value.data,
  //     'headers': value.headers.map,
  //   }));
  // }).catchError((e) {
  //   print(e);
  //   Map<String, dynamic> data = {};
  //   data = {
  //     'kind': 'Unknown',
  //     'url': map['url'],
  //     'map': e.toString(),
  //   };
  //   writeString(jsonEncode(data));
  // });

  return 0;
}

void logEvent(Pointer<Uint8> pointer, int len) {
  print("log event");
}

class Quelle {
  final String path;

  late EngineResource _engine;

  Quelle(this.path) {
    Pointer<Pointer<types.Engine>> engineOut = calloc();
    final pathC = Utf8Resource(path.toNativeUtf8());
    try {
      final result = bindings.open_engine_with_path(
        pathC.pointer(),
        path.length,
        Pointer.fromFunction(sendRequest, 0),
        Pointer.fromFunction(logEvent),
        engineOut,
      );
      if (result != 0) throw _readError(-result);
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
          bindings.fetch_novel(_engine.unsafe(), urlC.pointer(), url.length);
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
      final signedLength = bindings.fetch_chapter_content(
          _engine.unsafe(), urlC.pointer(), url.length);
      return MemLoc(_engine, signedLength).readAsString();
    } finally {
      urlC.free();
    }
  }

  bool popularSupported() {
    final result = bindings.popular_supported(_engine.unsafe());
    if (result < 0) throw _readError(-result);
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
    if (result < 0) throw _readError(-result);
    return result > 0;
  }

  String textSearchJson(String query, int page) {
    final queryC = Utf8Resource(query.toNativeUtf8());

    try {
      final signedLength = bindings.text_search(
          _engine.unsafe(), queryC.pointer(), query.length, page);
      return MemLoc(_engine, signedLength).readAsString();
    } finally {
      queryC.free();
    }
  }

  QuelleException _readError(int length) {
    final pointer = bindings.last_result();
    final errorMessage = pointer.toDartString(length: length);
    calloc.free(pointer);
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

  Pointer<Uint8> pointer() => _cString.cast();
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

void writeString(String value) {
  final pointer = value.toNativeUtf8Unterminated();
  bindings.set_last_offset(value.length);
  bindings.set_last_pointer(pointer.cast());
}

class QuelleException implements Exception {
  final String message;

  QuelleException(this.message);

  @override
  String toString() => "QuelleException: $message";
}
