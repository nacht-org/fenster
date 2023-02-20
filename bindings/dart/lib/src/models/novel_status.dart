import '../quelle.dart';

enum NovelStatus {
  ongoing,
  hiatus,
  completed,
  stub,
  unknown;

  factory NovelStatus.parse(String value) {
    switch (value.toLowerCase()) {
      case 'ongoing':
        return NovelStatus.ongoing;
      case 'hiatus':
        return NovelStatus.hiatus;
      case 'completed':
        return NovelStatus.completed;
      case 'stub':
        return NovelStatus.stub;
      case 'unknown':
        return NovelStatus.unknown;
      default:
        throw QuelleException("'$value' is not a valid novel status");
    }
  }
}
