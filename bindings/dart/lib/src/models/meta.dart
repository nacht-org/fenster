import 'attribute.dart';
import 'reading_direction.dart';

class Meta {
  const Meta({
    required this.id,
    required this.name,
    required this.langs,
    required this.version,
    required this.baseUrls,
    required this.readingDirections,
    required this.attributes,
  });

  final String id;
  final String name;
  final List<String> langs;
  final String version;
  final List<String> baseUrls;
  final List<ReadingDirection> readingDirections;
  final List<Attribute> attributes;

  factory Meta.parse(Map<String, dynamic> map) {
    return Meta(
      id: map['id'],
      name: map['name'],
      langs: (map['langs'] as List<dynamic>).whereType<String>().toList(),
      version: map['version'],
      baseUrls:
          (map['base_urls'] as List<dynamic>).whereType<String>().toList(),
      readingDirections: (map['rds'] as List<dynamic>)
          .whereType<String>()
          .map(ReadingDirection.parse)
          .toList(),
      attributes: (map['attrs'] as List<dynamic>)
          .whereType<String>()
          .map(Attribute.parse)
          .toList(),
    );
  }
}
