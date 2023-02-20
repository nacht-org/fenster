import '../quelle.dart';

class Metadata {
  Metadata({
    required this.name,
    required this.value,
    required this.namespace,
    required this.others,
  });

  final String name;
  final String value;
  final Namespace namespace;
  final Map<String, dynamic> others;

  factory Metadata.parse(Map<String, dynamic> map) {
    return Metadata(
      name: map['name'],
      value: map['value'],
      namespace: Namespace.parse(map['ns']),
      others: map['others'],
    );
  }
}

enum Namespace {
  dc,
  opf;

  factory Namespace.parse(String value) {
    switch (value.toLowerCase()) {
      case 'dc':
        return Namespace.dc;
      case 'opf':
        return Namespace.opf;
      default:
        throw QuelleException("'$value' is not a valid namespace");
    }
  }
}
