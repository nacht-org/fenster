import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings/bindings.dart';
import 'bindings/types.dart' as types;

class Quelle {
  final String path;

  late EngineResource _engine;

  Quelle(this.path) {
    Pointer<Pointer<types.Engine>> engineOut = calloc();
    final pathC = Utf8Resource(path.toNativeUtf8());
    final result = bindings.open_engine_with_path(pathC.unsafe(), engineOut);
    _engine = EngineResource(engineOut.value);
    calloc.free(engineOut);
    pathC.free();
  }

  void meta() {
    Pointer<Pointer<Utf8>> out = calloc();
    final result = bindings.source_meta(_engine.unsafe(), out);
    final meta = out.value.toDartString();
    calloc.free(out.value);
    calloc.free(out);
    print(result);
    print(meta);
  }
}

class EngineResource implements Finalizable {
  late Pointer<types.Engine> _engine;

  EngineResource(this._engine);

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
