import 'dart:ffi';
import 'dart:io';

/// Attempts to locate the Quelle dynamic library.
///
/// Throws [QuelleLocatorError] if the dynamic library could not be found.
DynamicLibrary loadDynamicLibrary() {
  DynamicLibrary? locate(String libName) {
    if (FileSystemEntity.isFileSync(libName)) {
      return DynamicLibrary.open(libName);
    }

    final toolLib =
        Directory.current.uri.resolve("$_quelleToolDir$libName").toFilePath();
    if (FileSystemEntity.isFileSync(toolLib)) {
      return DynamicLibrary.open(toolLib);
    }

    return null;
  }

  DynamicLibrary locateOrError(String libName) {
    final value = locate(libName);
    if (value != null) {
      return value;
    } else {
      throw QuelleLocatorError(
        'Quelle library not found. Did you run `$invocationString`?',
      );
    }
  }

  if (Platform.isIOS) {
    return DynamicLibrary.process();
  } else if (Platform.isMacOS) {
    return locateOrError(appleLib);
  } else if (Platform.isWindows) {
    return locate(windowsLib) ?? DynamicLibrary.executable();
  } else if (Platform.isLinux) {
    return locateOrError(linuxLib);
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

const _quelleToolDir = '.dart_tool/quelle/';
