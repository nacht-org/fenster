class Chapter {
  Chapter({
    required this.index,
    required this.title,
    required this.url,
    required this.updatedAt,
  });

  final int index;
  final String title;
  final String url;
  final DateTime? updatedAt;

  factory Chapter.parse(Map<String, dynamic> map) {
    final updatedAtString =
        map['updated_at']['Utc'] ?? map['updated_at']['Local'];

    // FIXME: handle local date times appropriately
    final updatedAt =
        updatedAtString != null ? DateTime.parse(updatedAtString) : null;

    return Chapter(
      index: map['index'],
      title: map['title'],
      url: map['url'],
      updatedAt: updatedAt,
    );
  }
}
