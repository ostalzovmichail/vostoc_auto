import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'constants/app_constants.dart';
import 'screens/home/home_screen.dart';
import 'screens/settings/settings_screen.dart';
import 'services/settings.dart';

void main() {
  runApp(
    const ProviderScope(
      child: BeregAdminApp(),
    ),
  );
}

/// Тема приложения: Material 3 (стиль Android) с фирменным синим оттенком.
ThemeData _theme(Brightness brightness) {
  return ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: AppConstants.seedColor,
      brightness: brightness,
    ),
  );
}

class BeregAdminApp extends ConsumerWidget {
  const BeregAdminApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(appSettingsProvider);

    return MaterialApp(
      title: AppConstants.appTitle,
      debugShowCheckedModeBanner: false,
      theme: _theme(Brightness.light),
      darkTheme: _theme(Brightness.dark),
      home: settings.isLoading
          ? const Scaffold(body: Center(child: CircularProgressIndicator()))
          : settings.isConfigured
              ? const HomeScreen()
              : const SettingsScreen(firstRun: true),
    );
  }
}
