import 'package:quelle/src/quelle.dart';

void main(List<String> args) {
  final quelle = Quelle("../../extensions/extension_novelpub.wasm");
  final meta = quelle.meta();
}
