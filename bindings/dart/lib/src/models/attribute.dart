import '../quelle.dart';

enum Attribute {
  fanfiction;

  factory Attribute.parse(String value) {
    switch (value.toLowerCase()) {
      case "fanfiction":
        return Attribute.fanfiction;
      default:
        throw QuelleException("'$value' is not a valid attribute");
    }
  }
}
