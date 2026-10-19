import 'package:flutter_test/flutter_test.dart';

import 'package:bereg_admin/models/car.dart';

void main() {
  group('Car.fromJson', () {
    test('читает полный ответ API', () {
      final car = Car.fromJson({
        'id': 15,
        'lot_number': '0249',
        'brand': 'Audi',
        'model': 'A3',
        'year': 2021,
        'mileage': 55200,
        'engine': '1.4T (150 л.с.)',
        'transmission': 'S tronic',
        'drive': '2WD',
        'price_rub': 1940000,
        'auction_price_yen': null,
        'auction_name': null,
        'auction_lot': null,
        'rating': null,
        'status': 'stock',
        'transit_days': null,
        'delivery_city': null,
        'image_url': '/images/audi-a3/audi-a3-1.jpg',
        'images': ['/images/audi-a3/audi-a3-1.jpg'],
        'created_at': '2026-10-16 10:00:00',
        'sold_at': null,
        'deal_duration_days': null,
      });

      expect(car.id, 15);
      expect(car.title, 'Audi A3');
      expect(car.statusEnum, CarStatus.stock);
      expect(car.coverPath, '/images/audi-a3/audi-a3-1.jpg');
      expect(car.images, hasLength(1));
    });

    test('не падает на отсутствующих необязательных полях', () {
      final car = Car.fromJson({'id': 1, 'brand': 'Lada'});

      expect(car.id, 1);
      expect(car.mileage, 0);
      expect(car.images, isEmpty);
      expect(car.coverPath, isNull);
    });

    test('неизвестный статус считается «в наличии»', () {
      expect(CarStatus.fromValue('что-то новое'), CarStatus.stock);
      expect(CarStatus.fromValue('sold'), CarStatus.sold);
    });
  });

  group('форматирование', () {
    test('цена разделяется по тысячам', () {
      expect(formatRub(1940000), '1 940 000 ₽');
      expect(formatRub(950), '950 ₽');
    });

    test('пробег разделяется по тысячам', () {
      expect(formatMileage(55200), '55 200 км');
    });
  });
}
