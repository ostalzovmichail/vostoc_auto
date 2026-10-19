import 'package:flutter/material.dart';

import '../../models/car.dart';

/// Цвет статуса автомобиля для чипа/подписи.
Color carStatusColor(CarStatus status, ThemeData theme) => switch (status) {
      CarStatus.stock => Colors.green.shade700,
      CarStatus.transit => Colors.blue.shade700,
      CarStatus.order => Colors.orange.shade800,
      CarStatus.sold => theme.colorScheme.outline,
    };

/// Компактный цветной чип со статусом автомобиля.
class StatusChip extends StatelessWidget {
  const StatusChip({super.key, required this.status});

  final CarStatus status;

  @override
  Widget build(BuildContext context) {
    final color = carStatusColor(status, Theme.of(context));
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Text(
        status.label,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
