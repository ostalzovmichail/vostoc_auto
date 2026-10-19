import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'settings.g.dart';

/// Настройки подключения к серверу автосалона.
///
/// Хранятся на устройстве (`shared_preferences`): приложение открывается сразу
/// на нужном сервере и не спрашивает токен при каждом запуске.
@riverpod
class AppSettings extends _$AppSettings {
  static const _baseUrlKey = 'base_url';
  static const _tokenKey = 'admin_token';

  /// Адрес сервера по умолчанию с учётом платформы.
  ///
  /// На Android-эмуляторе хост-машина доступна по адресу `10.0.2.2`, а не
  /// `127.0.0.1` (тот указывает на сам эмулятор) — подсказываем верный вариант.
  static String defaultBaseUrl() {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      return 'http://10.0.2.2:8080';
    }
    return 'http://127.0.0.1:8080';
  }

  @override
  AppSettingsState build() {
    // Начальное состояние — загрузка
    _load();
    return AppSettingsState.loading();
  }

  /// Читает сохранённые значения (вызывается один раз при запуске).
  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final baseUrl = prefs.getString(_baseUrlKey) ?? defaultBaseUrl();
    final token = prefs.getString(_tokenKey) ?? '';
    state = AppSettingsState.loaded(baseUrl: baseUrl, token: token);
  }

  /// Сохраняет адрес и токен.
  Future<void> save({required String baseUrl, required String token}) async {
    final cleanBaseUrl = baseUrl.trim().replaceFirst(RegExp(r'/+$'), '');
    final cleanToken = token.trim();

    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_baseUrlKey, cleanBaseUrl);
    await prefs.setString(_tokenKey, cleanToken);

    state = AppSettingsState.loaded(baseUrl: cleanBaseUrl, token: cleanToken);
  }

  /// Сбрасывает настройки (для тестов/логаута).
  Future<void> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_baseUrlKey);
    await prefs.remove(_tokenKey);
    state = AppSettingsState.loaded(baseUrl: defaultBaseUrl(), token: '');
  }
}

/// Состояние настроек подключения.
sealed class AppSettingsState {
  const AppSettingsState();

  /// Настройки загружаются.
  const factory AppSettingsState.loading() = _Loading;

  /// Настройки загружены.
  const factory AppSettingsState.loaded({
    required String baseUrl,
    required String token,
  }) = _Loaded;

  /// Настройки ещё загружаются с диска.
  bool get isLoading => this is _Loading;

  /// Готово ли приложение к работе: заданы адрес и токен.
  bool get isConfigured {
    return switch (this) {
      _Loaded(baseUrl: final b, token: final t) =>
        b.trim().isNotEmpty && t.trim().isNotEmpty,
      _Loading() => false,
    };
  }

  String get baseUrl {
    return switch (this) {
      _Loaded(baseUrl: final b) => b,
      _Loading() => '',
    };
  }

  String get token {
    return switch (this) {
      _Loaded(token: final t) => t,
      _Loading() => '',
    };
  }
}

class _Loading extends AppSettingsState {
  const _Loading();
}

class _Loaded extends AppSettingsState {
  const _Loaded({required this.baseUrl, required this.token});

  @override
  final String baseUrl;

  @override
  final String token;
}