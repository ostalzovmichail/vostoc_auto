import 'dart:convert';
import 'dart:typed_data';

import 'package:http/http.dart' as http;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../constants/api_constants.dart';
import '../models/car.dart';
import '../models/site_visits.dart';
import 'settings.dart';

part 'api_client.g.dart';

/// Ошибка обращения к API с сообщением, которое можно показать пользователю.
class ApiException implements Exception {
  ApiException(this.message);

  final String message;

  @override
  String toString() => message;
}

/// Клиент админ-API автосалона.
///
/// Все операции защищены заголовком `X-Admin-Token`. Адрес сервера и токен
/// приходят из настроек приложения, поэтому на одном устройстве можно
/// переключаться между локальным сервером и продом.
class ApiClient {
  ApiClient({required String baseUrl, required this.token})
      : baseUrl = baseUrl.trim().replaceFirst(RegExp(r'/+$'), '');

  /// Адрес сервера без хвостового слэша (например, `http://127.0.0.1:8080`).
  final String baseUrl;

  /// Токен администратора.
  final String token;

  final http.Client _client = http.Client();

  /// Таймаут запросов.
  static const Duration _timeout = ApiConstants.requestTimeout;

  Map<String, String> get _headers => {
        'X-Admin-Token': token,
        'Content-Type': 'application/json; charset=utf-8',
      };

  Uri _uri(String path, [Map<String, String>? query]) {
    final uri = Uri.parse('$baseUrl$path');
    if (query == null || query.isEmpty) return uri;
    return uri.replace(queryParameters: query);
  }

  /// Превращает относительный путь фото (`/images/...`) в полный адрес.
  String? imageUrl(String? path) {
    if (path == null || path.isEmpty) return null;
    if (path.startsWith('http://') || path.startsWith('https://')) return path;
    return '$baseUrl$path';
  }

  /// Список автомобилей. `status == null` — все, включая проданные.
  Future<List<Car>> listCars({String? status}) async {
    final query = <String, String>{'limit': '100'};
    if (status != null) query['status'] = status;

    final response = await _client
        .get(_uri('/api/admin/cars', query), headers: _headers)
        .timeout(_timeout);

    final data = _decode(response);
    if (data is! List) {
      throw ApiException('Неожиданный ответ сервера: ожидался список авто');
    }
    return data
        .whereType<Map<String, dynamic>>()
        .map(Car.fromJson)
        .toList(growable: false);
  }

  /// Добавляет автомобиль.
  Future<Car> createCar(CarDraft draft) async {
    final response = await _client
        .post(
          _uri('/api/admin/cars'),
          headers: _headers,
          body: jsonEncode(draft.toJson()),
        )
        .timeout(_timeout);
    return Car.fromJson(_decode(response) as Map<String, dynamic>);
  }

  /// Меняет статус автомобиля (в том числе переводит в «продано»).
  Future<Car> setStatus(
    int id,
    CarStatus status, {
    String? deliveryCity,
    int? transitDays,
  }) async {
    final payload = <String, dynamic>{'status': status.value};
    if (deliveryCity != null && deliveryCity.trim().isNotEmpty) {
      payload['delivery_city'] = deliveryCity.trim();
    }
    if (transitDays != null) {
      payload['transit_days'] = transitDays;
    }

    final response = await _client
        .patch(
          _uri('/api/admin/cars/$id/status'),
          headers: _headers,
          body: jsonEncode(payload),
        )
        .timeout(_timeout);
    return Car.fromJson(_decode(response) as Map<String, dynamic>);
  }

  /// Удаляет автомобиль вместе с его фотографиями.
  Future<void> deleteCar(int id) async {
    final response = await _client
        .delete(_uri('/api/admin/cars/$id'), headers: _headers)
        .timeout(_timeout);
    _decode(response);
  }

  /// Загружает фотографию автомобиля (multipart, поле `file`).
  ///
  /// Байты, а не путь: на web путь к файлу недоступен, а `withData: true`
  /// во `file_picker` отдаёт содержимое на всех платформах — один код на все.
  Future<Car> uploadImage(
    int id, {
    required String fileName,
    required Uint8List bytes,
  }) async {
    final request = http.MultipartRequest('POST', _uri('/api/admin/cars/$id/images'))
      ..headers['X-Admin-Token'] = token
      ..files.add(
        http.MultipartFile.fromBytes('file', bytes, filename: fileName),
      );

    final streamed = await request.send().timeout(_timeout);
    final response = await http.Response.fromStream(streamed);
    return Car.fromJson(_decode(response) as Map<String, dynamic>);
  }

  /// Счётчик посещений сайта. Доступен только админ-API.
  Future<SiteVisits> fetchSiteVisits() async {
    final response = await _client
        .get(_uri('/api/admin/stats/visits'), headers: _headers)
        .timeout(_timeout);
    return SiteVisits.fromJson(_decode(response) as Map<String, dynamic>);
  }

  /// Удаляет одно фото по его пути, который вернул сервер.
  Future<Car> deleteImage(int id, String url) async {
    final response = await _client
        .delete(
          _uri('/api/admin/cars/$id/images', {'url': url}),
          headers: _headers,
        )
        .timeout(_timeout);
    return Car.fromJson(_decode(response) as Map<String, dynamic>);
  }

  /// Разбирает ответ: успех → декодированное тело, ошибка → [ApiException].
  ///
  /// Сообщение берём из стандартного тела `{"message": "..."}` — его пишет
  /// бэкенд, поэтому пользователь видит причину на русском, а не «ошибка 400».
  dynamic _decode(http.Response response) {
    dynamic body;
    if (response.bodyBytes.isNotEmpty) {
      try {
        body = jsonDecode(utf8.decode(response.bodyBytes));
      } on FormatException {
        body = null;
      }
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return body;
    }

    final message = (body is Map && body['message'] is String)
        ? body['message'] as String
        : 'Ошибка запроса (${response.statusCode})';
    throw ApiException(message);
  }
}

/// Провайдер ApiClient — пересоздаётся при смене настроек.
@riverpod
ApiClient apiClient(Ref ref) {
  final settings = ref.watch(appSettingsProvider);
  return ApiClient(baseUrl: settings.baseUrl, token: settings.token);
}
