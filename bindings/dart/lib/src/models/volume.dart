import 'package:quelle/src/models/chapter.dart';

class Volume {
  const Volume({
    required this.index,
    required this.name,
    required this.chapters,
  });

  final int index;
  final String name;
  final List<Chapter> chapters;

  factory Volume.parse(Map<String, dynamic> map) {
    return Volume(
      index: map['index'],
      name: map['name'],
      chapters: (map['chapters'] as List<dynamic>)
          .whereType<Map<String, dynamic>>()
          .map(Chapter.parse)
          .toList(),
    );
  }
}
