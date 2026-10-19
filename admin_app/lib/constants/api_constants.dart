/// Константы для сетевого взаимодействия.
class ApiConstants {
  ApiConstants._();

  /// Таймаут запроса: без него «зависший» сервер подвесит весь интерфейс.
  static const Duration requestTimeout = Duration(seconds: 20);
}
