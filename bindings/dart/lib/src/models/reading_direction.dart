import '../quelle.dart';

enum ReadingDirection {
  ltr,
  rtl;

  factory ReadingDirection.parse(String value) {
    switch (value.toLowerCase()) {
      case "ltr":
        return ReadingDirection.ltr;
      case "rtl":
        return ReadingDirection.rtl;
      default:
        throw QuelleException("'$value' is not a valid reading direction");
    }
  }
}
