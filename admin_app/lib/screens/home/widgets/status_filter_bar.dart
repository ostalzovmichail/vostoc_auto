import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../models/car.dart';
import '../../../providers/cars_providers.dart';

/// Горизонтальный ряд чипов-фильтров по статусу автомобиля.
class StatusFilterBar extends ConsumerWidget {
  const StatusFilterBar({super.key, required this.selected});

  /// Текущий выбранный статус; `null` — без фильтра (все, включая проданные).
  final CarStatus? selected;

  // Пары «статус — подпись»: `null` — без фильтра.
  static const _options = <(CarStatus?, String)>[
    (null, 'Все'),
    (CarStatus.stock, 'В наличии'),
    (CarStatus.transit, 'В пути'),
    (CarStatus.order, 'Под заказ'),
    (CarStatus.sold, 'Продано'),
  ];

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return SizedBox(
      height: 56,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        itemCount: _options.length,
        separatorBuilder: (_, _) => const SizedBox(width: 8),
        itemBuilder: (context, index) {
          final (status, label) = _options[index];
          return ChoiceChip(
            label: Text(label),
            selected: selected == status,
            onSelected: (_) =>
                ref.read(carStatusFilterProvider.notifier).set(status),
          );
        },
      ),
    );
  }
}
