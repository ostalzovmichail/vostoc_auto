/// Статусы автомобиля — ровно те значения, что принимает админ-API бэкенда.
///
/// Держим их синхронно с `CAR_STATUSES` в `bec/src/core/models.rs`: если там
/// появится новый статус, его нужно добавить и здесь.
enum CarStatus {
  stock('stock', 'В наличии'),
  transit('transit', 'В пути'),
  order('order', 'Под заказ'),
  sold('sold', 'Продано');

  const CarStatus(this.value, this.label);

  /// Значение, которое уходит в API.
  final String value;

  /// Подпись для интерфейса.
  final String label;

  /// Разбирает значение из API; неизвестное считаем «в наличии», чтобы
  /// список не падал из-за одной новой записи на сервере.
  static CarStatus fromValue(String value) => CarStatus.values.firstWhere(
        (status) => status.value == value,
        orElse: () => CarStatus.stock,
      );
}

/// Автомобиль из каталога (ответ админ-API).
class Car {
  const Car({
    required this.id,
    required this.lotNumber,
    required this.brand,
    required this.model,
    required this.year,
    required this.mileage,
    required this.engine,
    required this.transmission,
    required this.drive,
    required this.priceRub,
    required this.status,
    required this.images,
    required this.createdAt,
    this.auctionPriceYen,
    this.auctionName,
    this.auctionLot,
    this.rating,
    this.transitDays,
    this.deliveryCity,
    this.imageUrl,
    this.soldAt,
    this.dealDurationDays,
  });

  final int id;
  final String lotNumber;
  final String brand;
  final String model;
  final int year;
  final int mileage;
  final String engine;
  final String transmission;
  final String drive;
  final int priceRub;
  final String status;
  final List<String> images;
  final String createdAt;
  final int? auctionPriceYen;
  final String? auctionName;
  final String? auctionLot;
  final String? rating;
  final int? transitDays;
  final String? deliveryCity;
  final String? imageUrl;
  final String? soldAt;
  final int? dealDurationDays;

  /// Статус в виде enum — для подписей и логики интерфейса.
  CarStatus get statusEnum => CarStatus.fromValue(status);

  /// «Audi A3» — короткое имя для списков.
  String get title => '$brand $model';

  /// Первое фото галереи, если обложка не задана явно.
  String? get coverPath =>
      imageUrl ?? (images.isNotEmpty ? images.first : null);

  factory Car.fromJson(Map<String, dynamic> json) {
    return Car(
      id: _asInt(json['id']),
      lotNumber: _asString(json['lot_number']),
      brand: _asString(json['brand']),
      model: _asString(json['model']),
      year: _asInt(json['year']),
      mileage: _asInt(json['mileage']),
      engine: _asString(json['engine']),
      transmission: _asString(json['transmission']),
      drive: _asString(json['drive']),
      priceRub: _asInt(json['price_rub']),
      status: _asString(json['status']),
      images: (json['images'] as List<dynamic>? ?? const [])
          .map((item) => item.toString())
          .toList(growable: false),
      createdAt: _asString(json['created_at']),
      auctionPriceYen: _asIntOrNull(json['auction_price_yen']),
      auctionName: _asStringOrNull(json['auction_name']),
      auctionLot: _asStringOrNull(json['auction_lot']),
      rating: _asStringOrNull(json['rating']),
      transitDays: _asIntOrNull(json['transit_days']),
      deliveryCity: _asStringOrNull(json['delivery_city']),
      imageUrl: _asStringOrNull(json['image_url']),
      soldAt: _asStringOrNull(json['sold_at']),
      dealDurationDays: _asIntOrNull(json['deal_duration_days']),
    );
  }

  /// Аккуратные приведения JSON → Dart: сервер может вернуть null там, где
  /// поле описано как необязательное, и падать из-за этого не нужно.
  static int _asInt(dynamic value) => _asIntOrNull(value) ?? 0;

  static int? _asIntOrNull(dynamic value) {
    if (value is int) return value;
    if (value is num) return value.toInt();
    if (value is String) return int.tryParse(value);
    return null;
  }

  static String _asString(dynamic value) => value?.toString() ?? '';

  static String? _asStringOrNull(dynamic value) {
    final text = value?.toString();
    return (text == null || text.isEmpty) ? null : text;
  }
}

/// Тело запроса на создание автомобиля (`POST /api/admin/cars`).
///
/// Отдельный класс, а не «голый» Map: так видно обязательные поля и нельзя
/// случайно отправить лишний ключ.
class CarDraft {
  const CarDraft({
    required this.brand,
    required this.model,
    required this.year,
    required this.mileage,
    required this.engine,
    required this.transmission,
    required this.drive,
    required this.priceRub,
    this.lotNumber,
    this.status = CarStatus.stock,
  });

  final String brand;
  final String model;
  final int year;
  final int mileage;
  final String engine;
  final String transmission;
  final String drive;
  final int priceRub;
  final String? lotNumber;
  final CarStatus status;

  Map<String, dynamic> toJson() => {
        'brand': brand.trim(),
        'model': model.trim(),
        'year': year,
        'mileage': mileage,
        'engine': engine.trim(),
        'transmission': transmission.trim(),
        'drive': drive.trim(),
        'price_rub': priceRub,
        'status': status.value,
        if (lotNumber != null && lotNumber!.trim().isNotEmpty)
          'lot_number': lotNumber!.trim(),
      };
}

/// «1 940 000 ₽» — разделяем тысячи пробелом, как принято в объявлениях.
String formatRub(int value) {
  final digits = value.abs().toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) buffer.write(' ');
    buffer.write(digits[i]);
  }
  return '${value < 0 ? '-' : ''}$buffer ₽';
}

/// «55 200 км» — пробег с разделителем тысяч.
String formatMileage(int value) {
  final digits = value.abs().toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) buffer.write(' ');
    buffer.write(digits[i]);
  }
  return '$buffer км';
}
