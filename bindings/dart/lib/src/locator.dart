import 'dart:ffi';
import 'dart:io';

/// Attempts to locate the Quelle dynamic library.
///
/// Throws [QuelleLocatorError] if the dynamic library could not be found.
DynamicLibrary loadDynamicLibrary() {
  String locate(String libName) {
    final dir = '$_wasmToolDir$libName';
    final value = _packageRootUri(Platform.script.resolve('./')) ??
        _packageRootUri(Directory.current.uri);
    if (value != null) {
      return value.resolve(dir).toFilePath();
    } else {
      throw QuelleLocatorError(
        'Wasm library not found. Did you run `$invocationString`?',
      );
    }
  }

  if (Platform.isIOS) {
    return DynamicLibrary.process();
  } else if (Platform.isMacOS) {
    return DynamicLibrary.open(locate(appleLib));
  } else if (Platform.isWindows) {
    return DynamicLibrary.open(locate(windowsLib));
  } else if (Platform.isLinux) {
    return DynamicLibrary.open(locate(linuxLib));
  } else if (Platform.isAndroid) {
    return DynamicLibrary.open(linuxLib);
  } else if (Platform.isFuchsia) {
    throw QuelleLocatorError(
      'Quelle is currently not supported on Fuchsia.',
    );
  } else {
    throw QuelleLocatorError(
      'Quelle is currently not supported on this platform.',
    );
  }
}

/// This error is thrown when the dynamic library could not be found.
class QuelleLocatorError extends Error {
  final String message;

  QuelleLocatorError(
    this.message,
  );

  @override
  String toString() => 'QuelleLocatorError: $message';
}

/// The command that can be used to set up this package.
const invocationString = 'dart run quelle:setup';

/// The expected name of the Quelle library when compiled for Apple devices.
const appleLib = 'libquelle.dylib';

/// The expected name of the Quelle library when compiled for Linux devices.
const linuxLib = 'libquelle.so';

/// The expected name of the Quelle library when compiled for Windows devices.
const windowsLib = 'quelle.dll';

/// Returns the uri representing the target output directory of generated
/// dynamic libraries.
Uri libBuildOutDir(Uri root) {
  final pkgRoot = _packageRootUri(root);
  if (pkgRoot == null) {
    throw ArgumentError('Could not find "$_pkgConfigFile" within "$root".');
  }
  return pkgRoot.resolve(_wasmToolDir);
}

const _wasmToolDir = '.dart_tool/wasm/';

const _pkgConfigFile = '.dart_tool/package_config.json';

Uri? _packageRootUri(Uri root) {
  do {
    if (FileSystemEntity.isFileSync(
      root.resolve(_pkgConfigFile).toFilePath(),
    )) {
      return root;
    }
  } while (root != (root = root.resolve('..')));
  return null;
}
