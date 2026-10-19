import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../models/car.dart';
import '../../../services/api_client.dart';
import '../../../widgets/car_image.dart';
import '../../../widgets/status_chip.dart';

/// Действия из контекстного меню карточки автомобиля.
enum CarCardAction { detail, sold, delete }

/// Карточка автомобиля в списке каталога.
class CarCard extends ConsumerWidget {
  const CarCard({
    super.key,
    required this.car,
    required this.onOpen,
    required this.onMarkSold,
    required this.onDelete,
  });

  final Car car;
  final ValueChanged<Car> onOpen;
  final ValueChanged<Car> onMarkSold;
  final ValueChanged<Car> onDelete;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final api = ref.read(apiClientProvider);
    final cover = api.imageUrl(car.coverPath);

    return Card(
      clipBehavior: Clip.antiAlias,
      margin: const EdgeInsets.symmetric(vertical: 6),
      child: InkWell(
        onTap: () => onOpen(car),
        child: Padding(
          padding: const EdgeInsets.all(10),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              ClipRRect(
                borderRadius: BorderRadius.circular(10),
                child: CarImage(url: cover, width: 92, height: 72),
              ),
              const SizedBox(width: 12),
              Expanded(child: _CarInfo(car: car, theme: theme)),
              _actionsMenu(context),
            ],
          ),
        ),
      ),
    );
  }

  Widget _actionsMenu(BuildContext context) {
    return PopupMenuButton<CarCardAction>(
      tooltip: 'Действия',
      onSelected: (action) {
        switch (action) {
          case CarCardAction.detail:
            onOpen(car);
          case CarCardAction.sold:
            onMarkSold(car);
          case CarCardAction.delete:
            onDelete(car);
        }
      },
      itemBuilder: (context) => [
        const PopupMenuItem(
          value: CarCardAction.detail,
          child: _MenuRow(icon: Icons.info_outline, label: 'Подробнее'),
        ),
        if (car.statusEnum != CarStatus.sold)
          const PopupMenuItem(
            value: CarCardAction.sold,
            child: _MenuRow(icon: Icons.sell_outlined, label: 'Отметить проданным'),
          ),
        const PopupMenuItem(
          value: CarCardAction.delete,
          child: _MenuRow(icon: Icons.delete_outline, label: 'Удалить'),
        ),
      ],
    );
  }
}

/// Текстовая часть карточки: название, характеристики и цена.
class _CarInfo extends StatelessWidget {
  const _CarInfo({required this.car, required this.theme});

  final Car car;
  final ThemeData theme;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                car.title,
                overflow: TextOverflow.ellipsis,
                style: theme.textTheme.titleMedium,
              ),
            ),
            StatusChip(status: car.statusEnum),
          ],
        ),
        const SizedBox(height: 4),
        Text(
          'Лот ${car.lotNumber} · ${car.year} · ${car.transmission}',
          style: theme.textTheme.bodySmall
              ?.copyWith(color: theme.colorScheme.outline),
        ),
        Text(
          formatMileage(car.mileage),
          style: theme.textTheme.bodySmall
              ?.copyWith(color: theme.colorScheme.outline),
        ),
        const SizedBox(height: 6),
        Text(
          formatRub(car.priceRub),
          style: theme.textTheme.titleSmall
              ?.copyWith(fontWeight: FontWeight.w700),
        ),
      ],
    );
  }
}

/// Строка пункта меню: иконка + подпись с отступом.
class _MenuRow extends StatelessWidget {
  const _MenuRow({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 20),
        const SizedBox(width: 12),
        Text(label),
      ],
    );
  }
}
