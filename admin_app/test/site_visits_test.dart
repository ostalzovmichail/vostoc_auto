import 'package:flutter_test/flutter_test.dart';

import 'package:bereg_admin/models/site_visits.dart';

void main() {
  group('SiteVisits.fromJson', () {
    test('читает счётчик из ответа админ-API', () {
      final visits = SiteVisits.fromJson({
        'total': 128,
        'today': 7,
        'last_7_days': 42,
      });

      expect(visits.total, 128);
      expect(visits.today, 7);
      expect(visits.last7Days, 42);
    });

    test('не падает на отсутствующих полях', () {
      final visits = SiteVisits.fromJson(const {});

      expect(visits.total, 0);
      expect(visits.today, 0);
      expect(visits.last7Days, 0);
    });

    test('приводит числовые строки', () {
      final visits = SiteVisits.fromJson({
        'total': '10',
        'today': '2',
        'last_7_days': 3.0,
      });

      expect(visits.total, 10);
      expect(visits.today, 2);
      expect(visits.last7Days, 3);
    });
  });
}
