import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../services/api_client.dart';
import '../../services/settings.dart';
import '../../constants/text_constants.dart';
import '../../utils/ui_feedback.dart';

/// Экран настроек подключения: адрес сервера и токен администратора.
class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key, this.firstRun = false});

  /// Первый запуск: экран нельзя закрыть (нечего показывать под ним).
  final bool firstRun;

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  late final TextEditingController _baseUrlController;
  late final TextEditingController _tokenController;
  bool _testing = false;
  bool _obscureToken = true;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(appSettingsProvider);
    _baseUrlController = TextEditingController(text: settings.baseUrl);
    _tokenController = TextEditingController(text: settings.token);
  }

  @override
  void dispose() {
    _baseUrlController.dispose();
    _tokenController.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    await ref
        .read(appSettingsProvider.notifier)
        .save(baseUrl: _baseUrlController.text, token: _tokenController.text);
    if (!mounted) return;
    showMessage(context, SettingsTexts.savedMessage);
    // При первом запуске корневой экран сам сменится на каталог.
    if (!widget.firstRun) Navigator.of(context).pop();
  }

  /// Проверяет связь: забирает список авто с текущими адресом и токеном.
  Future<void> _testConnection() async {
    setState(() => _testing = true);
    final api = ApiClient(
      baseUrl: _baseUrlController.text,
      token: _tokenController.text,
    );

    try {
      final cars = await api.listCars();
      if (!mounted) return;
      showMessage(context, '${SettingsTexts.connectionSuccess}${cars.length} авто');
    } on ApiException catch (error) {
      if (!mounted) return;
      showMessage(context, error.message, isError: true);
    } catch (error) {
      if (!mounted) return;
      showMessage(context, '${SettingsTexts.connectionError}$error', isError: true);
    } finally {
      if (mounted) setState(() => _testing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.firstRun ? SettingsTexts.firstRunTitle : SettingsTexts.normalTitle),
        automaticallyImplyLeading: !widget.firstRun,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(SettingsTexts.serverTitle, style: theme.textTheme.titleMedium),
                  const SizedBox(height: 8),
                  Text(
                    SettingsTexts.serverDescription,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.outline,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _baseUrlController,
            keyboardType: TextInputType.url,
            autocorrect: false,
            decoration: const InputDecoration(
              labelText: SettingsTexts.baseUrlLabel,
              hintText: SettingsTexts.baseUrlHint,
              prefixIcon: Icon(Icons.dns_outlined),
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _tokenController,
            obscureText: _obscureToken,
            autocorrect: false,
            enableSuggestions: false,
            decoration: InputDecoration(
              labelText: SettingsTexts.tokenLabel,
              prefixIcon: const Icon(Icons.key_outlined),
              border: const OutlineInputBorder(),
              suffixIcon: IconButton(
                tooltip: _obscureToken ? SettingsTexts.tokenShow : SettingsTexts.tokenHide,
                icon: Icon(
                  _obscureToken
                      ? Icons.visibility_outlined
                      : Icons.visibility_off_outlined,
                ),
                onPressed: () => setState(() => _obscureToken = !_obscureToken),
              ),
            ),
          ),
          const SizedBox(height: 20),
          FilledButton.icon(
            onPressed: _testing ? null : _testConnection,
            icon: _testing
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.wifi_tethering),
            label: const Text(SettingsTexts.testConnection),
          ),
          const SizedBox(height: 8),
          FilledButton.icon(
            onPressed: _save,
            icon: const Icon(Icons.save_outlined),
            label: const Text(SettingsTexts.enter),
          ),
        ],
      ),
    );
  }
}
