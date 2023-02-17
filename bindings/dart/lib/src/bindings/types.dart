import 'dart:ffi';

import 'package:ffi/ffi.dart';

class Engine extends Opaque {}

typedef StringBuffer = Pointer<Pointer<Utf8>>;
