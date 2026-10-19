import 'package:flutter/material.dart';

import '../../../models/car.dart';

/// Карточка с характеристиками автомобиля.
class SpecsCard extends StatelessWidget {
  const SpecsCard({super.key, required this.car});

  final Car car;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: Column(
          children: [
            InfoRow(label: 'Лот', value: car.lotNumber),
            InfoRow(label: 'Год', value: '${car.year}'),
            InfoRow(label: 'Пробег', value: formatMileage(car.mileage)),
            InfoRow(label: 'Двигатель', value: car.engine),
            InfoRow(label: 'Коробка передач', value: car.transmission),
            InfoRow(label: 'Привод', value: car.drive),
            if (car.auctionName != null)
              InfoRow(label: 'Аукцион', value: car.auctionName!),
            if (car.rating != null) InfoRow(label: 'Оценка', value: car.rating!),
            if (car.transitDays != null)
              InfoRow(label: 'Срок доставки', value: '${car.transitDays} дн.'),
            if (car.deliveryCity != null)
              InfoRow(label: 'Город доставки', value: car.deliveryCity!),
            InfoRow(label: 'Добавлен', value: car.createdAt),
            if (car.soldAt != null) InfoRow(label: 'Продан', value: car.soldAt!),
            if (car.dealDurationDays != null)
              InfoRow(
                label: 'Срок сделки',
                value: '${car.dealDurationDays} дн.',
              ),
          ],
        ),
      ),
    );
  }
}

/// Строка «подпись — значение» в карточке характеристик.
class InfoRow extends StatelessWidget {
  const InfoRow({super.key, required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 150,
            child: Text(
              label,
              style: theme.textTheme.bodyMedium
                  ?.copyWith(color: theme.colorScheme.outline),
            ),
          ),
          Expanded(
            child: Text(value, style: theme.textTheme.bodyMedium),
          ),
        ],
      ),
    );
  }
}
