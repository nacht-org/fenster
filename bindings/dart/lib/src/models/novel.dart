import 'models.dart';

class Novel {
  const Novel({
    required this.title,
    required this.authors,
    required this.url,
    required this.cover,
    required this.description,
    required this.status,
    required this.volumes,
    required this.metadata,
    required this.langs,
  });

  final String title;
  final List<String> authors;
  final String url;
  final String? cover;
  final List<String> description;
  final NovelStatus status;
  final List<Volume> volumes;
  final List<Metadata> metadata;
  final List<String> langs;

  factory Novel.parse(Map<String, dynamic> map) {
    return Novel(
      title: map['title'],
      authors: (map['authors'] as List<dynamic>).whereType<String>().toList(),
      url: map['url'],
      cover: map['cover'],
      description:
          (map['description'] as List<dynamic>).whereType<String>().toList(),
      status: NovelStatus.parse(map['status']),
      volumes: (map['volumes'] as List<dynamic>)
          .whereType<Map<String, dynamic>>()
          .map(Volume.parse)
          .toList(),
      metadata: (map['metadata'] as List<dynamic>)
          .whereType<Map<String, dynamic>>()
          .map(Metadata.parse)
          .toList(),
      langs: (map['langs'] as List<dynamic>).whereType<String>().toList(),
    );
  }
}
