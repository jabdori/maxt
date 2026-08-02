import 'package:test/test.dart';

import '../lib/src/generated_contract.dart';
import '../lib/src/models.dart';

void main() {
  test('generated exchange and feature inventories match public models', () {
    expect(exchanges, Exchange.values.map((value) => value.id));
    expect(features, Feature.values.map((value) => value.wireName));
  });
}
