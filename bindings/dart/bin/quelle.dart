import 'package:quelle/quelle.dart';

void main(List<String> args) {
  final quelle = Quelle("../../extensions/extension_novelpub.wasm");
  final meta = quelle.meta();
  print(
      "Meta (id=${meta.id}, name=${meta.name}, version=${meta.version}, baseUrls:${meta.baseUrls})");

  // final novel = quelle.fetchNovel(
  //     "https://www.novelpub.com/novel/the-villains-side-of-the-novel-10021223");
  // final chapterCount = novel.volumes
  //     .map((e) => e.chapters)
  //     .reduce(((value, element) => [...value, ...element]))
  //     .length;
  // print(
  //     "Novel (title=${novel.title}, authors=${novel.authors}, cover=${novel.cover}, chapters: $chapterCount)");

  // final content = quelle.fetchChapterContent(
  //     "https://www.novelpub.com/novel/the-villains-side-of-the-novel-1495/chapter-1");
  // print(content);

  final popularSupported = quelle.popularSupported();
  print("popularSupported=$popularSupported");

  // if (popularSupported) {
  //   final popularNovels = quelle.popularJson(1);
  //   print(popularNovels);
  // }

  // final textSearchSupported = quelle.textSearchSupported();
  // print("textSearchSupported=$textSearchSupported");

  // if (textSearchSupported) {
  //   final textSearchResults = quelle.textSearchJson("solo", 1);
  //   print(textSearchResults);
  // }
}
