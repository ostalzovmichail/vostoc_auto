// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'settings.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Настройки подключения к серверу автосалона.
///
/// Хранятся на устройстве (`shared_preferences`): приложение открывается сразу
/// на нужном сервере и не спрашивает токен при каждом запуске.

@ProviderFor(AppSettings)
final appSettingsProvider = AppSettingsProvider._();

/// Настройки подключения к серверу автосалона.
///
/// Хранятся на устройстве (`shared_preferences`): приложение открывается сразу
/// на нужном сервере и не спрашивает токен при каждом запуске.
final class AppSettingsProvider
    extends $NotifierProvider<AppSettings, AppSettingsState> {
  /// Настройки подключения к серверу автосалона.
  ///
  /// Хранятся на устройстве (`shared_preferences`): приложение открывается сразу
  /// на нужном сервере и не спрашивает токен при каждом запуске.
  AppSettingsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'appSettingsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$appSettingsHash();

  @$internal
  @override
  AppSettings create() => AppSettings();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(AppSettingsState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<AppSettingsState>(value),
    );
  }
}

String _$appSettingsHash() => r'5aaebebf1511c1a512e46b818919d72bc950e80f';

/// Настройки подключения к серверу автосалона.
///
/// Хранятся на устройстве (`shared_preferences`): приложение открывается сразу
/// на нужном сервере и не спрашивает токен при каждом запуске.

abstract class _$AppSettings extends $Notifier<AppSettingsState> {
  AppSettingsState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AppSettingsState, AppSettingsState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AppSettingsState, AppSettingsState>,
              AppSettingsState,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
