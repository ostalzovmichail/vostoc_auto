import 'package:flutter/material.dart';

import '../../../models/car.dart';

/// Карточка с ценой «под ключ» и (опционально) ценой на аукционе в йенах.
class PriceCard extends StatelessWidget {
  const PriceCard({super.key, required this.car});

  final Car car;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Цена «под ключ»', style: theme.textTheme.bodySmall),
                Text(
                  formatRub(car.priceRub),
                  style: theme.textTheme.headlineSmall
                      ?.copyWith(fontWeight: FontWeight.w700),
                ),
              ],
            ),
            if (car.auctionPriceYen != null)
              Text(
                '${car.auctionPriceYen} ¥ на аукционе',
                style: theme.textTheme.bodySmall
                    ?.copyWith(color: theme.colorScheme.outline),
              ),
          ],
        ),
      ),
    );
  }
}
