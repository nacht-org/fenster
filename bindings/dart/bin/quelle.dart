import 'package:quelle/src/quelle.dart';

void main(List<String> args) {
  final quelle = Quelle("../../extensions/extension_novelpub.wasm");
  final meta = quelle.meta();
  print(meta);

  final novel = quelle.fetchNovelJson(
      "https://www.novelpub.com/novel/the-villains-side-of-the-novel-10021223");
  print(novel);
}
