import 'package:flutter/material.dart';

/// Пустое/ошибочное состояние списка.
///
/// Специально построено на `ListView`, а не `Center`: так pull-to-refresh
/// продолжает работать и на пустом экране.
class MessageView extends StatelessWidget {
  const MessageView({
    super.key,
    required this.title,
    required this.subtitle,
    required this.icon,
  });

  final String title;
  final String subtitle;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return ListView(
      physics: const AlwaysScrollableScrollPhysics(),
      children: [
        const SizedBox(height: 72),
        Icon(icon, size: 56, color: theme.colorScheme.outline),
        const SizedBox(height: 16),
        Center(child: Text(title, style: theme.textTheme.titleMedium)),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 32),
          child: Text(
            subtitle,
            textAlign: TextAlign.center,
            style: theme.textTheme.bodySmall
                ?.copyWith(color: theme.colorScheme.outline),
          ),
        ),
      ],
    );
  }
}
