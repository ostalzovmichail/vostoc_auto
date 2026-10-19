/// Счётчик посещений сайта — ответ админ-API (`/api/admin/stats/visits`).
///
/// Числа видит только администратор: на публичной странице счётчик не
/// показывается, а в публичном `/api/*` этих данных нет.
class SiteVisits {
  const SiteVisits({
    required this.total,
    required this.today,
    required this.last7Days,
  });

  /// Всего просмотров главной страницы за всё время.
  final int total;

  /// Просмотры за сегодня (по времени Владивостока).
  final int today;

  /// Просмотры за последние 7 дней, включая сегодняшний.
  final int last7Days;

  factory SiteVisits.fromJson(Map<String, dynamic> json) {
    return SiteVisits(
      total: _asInt(json['total']),
      today: _asInt(json['today']),
      last7Days: _asInt(json['last_7_days']),
    );
  }

  /// Безопасное приведение JSON → int: сервер не должен ронять UI
  /// неожиданным типом или `null`.
  static int _asInt(dynamic value) {
    if (value is int) return value;
    if (value is num) return value.toInt();
    if (value is String) return int.tryParse(value) ?? 0;
    return 0;
  }
}
