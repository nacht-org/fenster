import 'package:quelle/quelle.dart';

void main(List<String> args) {
  final quelle = Quelle("../../extensions/extension_royalroad.wasm");
  final meta = quelle.meta();
  print(
      "Meta (id=${meta.id}, name=${meta.name}, version=${meta.version}, baseUrls:${meta.baseUrls})");

  final novel = quelle.fetchNovel(
      "https://www.royalroad.com/fiction/62929/trinity-of-magic-progression-fantasy");
  final chapterCount = novel.volumes
      .map((e) => e.chapters)
      .reduce(((value, element) => [...value, ...element]))
      .length;
  print(
      "Novel (title=${novel.title}, authors=${novel.authors}, cover=${novel.cover}, chapters: $chapterCount)");

  final content = quelle.fetchChapterContent(
      "https://www.royalroad.com/fiction/62929/trinity-of-magic-progression-fantasy/chapter/1082056/chapter-1-leaving-home-i");
  print(content);

  final popularSupported = quelle.popularSupported();
  print("popularSupported=$popularSupported");

  final popularUrl = quelle.popularUrl(1);
  print("popularUrl=$popularUrl");

  if (popularSupported) {
    final popularNovels = quelle.popularJson(1);
    print(popularNovels);
  }

  final textSearchSupported = quelle.textSearchSupported();
  print("textSearchSupported=$textSearchSupported");

  if (textSearchSupported) {
    final textSearchResults = quelle.textSearchJson("solo", 1);
    print(textSearchResults);
  }
}
