import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../constants/text_constants.dart';
import '../../../models/site_visits.dart';
import '../../../services/api_client.dart';

/// Диалог со счётчиком посещений сайта.
///
/// Числа приходят только из админ-API — на публичной странице счётчика нет.
/// Данные разовые и живут, пока открыт диалог, поэтому состояние локальное,
/// без отдельного провайдера (тот же приём, что у «Проверить связь» в
/// настройках).
class VisitsDialog extends ConsumerStatefulWidget {
  const VisitsDialog({super.key});

  @override
  ConsumerState<VisitsDialog> createState() => _VisitsDialogState();
}

class _VisitsDialogState extends ConsumerState<VisitsDialog> {
  late Future<SiteVisits> _visits;

  @override
  void initState() {
    super.initState();
    _visits = ref.read(apiClientProvider).fetchSiteVisits();
  }

  void _reload() {
    setState(() => _visits = ref.read(apiClientProvider).fetchSiteVisits());
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return AlertDialog(
      title: const Text(VisitsTexts.title),
      content: SizedBox(
        width: 320,
        child: FutureBuilder<SiteVisits>(
          future: _visits,
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.waiting) {
              return const Padding(
                padding: EdgeInsets.symmetric(vertical: 24),
                child: Center(child: CircularProgressIndicator()),
              );
            }
            if (snapshot.hasError) {
              return Text(
                '${VisitsTexts.loadError}${snapshot.error}',
                style: TextStyle(color: theme.colorScheme.error),
              );
            }

            final visits = snapshot.data!;
            return Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                _VisitsRow(label: VisitsTexts.total, value: visits.total),
                _VisitsRow(label: VisitsTexts.today, value: visits.today),
                _VisitsRow(label: VisitsTexts.last7Days, value: visits.last7Days),
              ],
            );
          },
        ),
      ),
      actions: [
        IconButton(
          tooltip: VisitsTexts.refresh,
          icon: const Icon(Icons.refresh),
          onPressed: _reload,
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text(VisitsTexts.close),
        ),
      ],
    );
  }
}

/// Строка «подпись — значение» в диалоге посещений.
class _VisitsRow extends StatelessWidget {
  const _VisitsRow({required this.label, required this.value});

  final String label;
  final int value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: theme.textTheme.bodyLarge),
          Text(
            '$value',
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}
