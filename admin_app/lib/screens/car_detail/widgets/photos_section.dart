import 'package:flutter/material.dart';

import '../../../widgets/car_image.dart';

/// Секция фотографий автомобиля: заголовок, кнопка добавления, прогресс и
/// сетка миниатюр.
class PhotosSection extends StatelessWidget {
  const PhotosSection({
    super.key,
    required this.imageUrls,
    required this.busy,
    required this.uploadStatus,
    required this.onAdd,
    required this.onOpen,
    required this.onDelete,
  });

  /// Полные адреса фотографий.
  final List<String> imageUrls;

  /// Идёт операция — блокируем кнопки.
  final bool busy;

  /// Текст прогресса загрузки, `null` — ничего не грузится.
  final String? uploadStatus;

  final VoidCallback onAdd;
  final ValueChanged<String> onOpen;
  final ValueChanged<String> onDelete;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _header(theme),
        if (uploadStatus != null) ...[
          const SizedBox(height: 4),
          _progress(theme, uploadStatus!),
        ],
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final url in imageUrls)
              PhotoTile(
                url: url,
                busy: busy,
                onOpen: () => onOpen(url),
                onDelete: () => onDelete(url),
              ),
            if (imageUrls.isEmpty)
              Text(
                'Фотографий пока нет — добавьте первое фото.',
                style: theme.textTheme.bodySmall
                    ?.copyWith(color: theme.colorScheme.outline),
              ),
          ],
        ),
      ],
    );
  }

  Widget _header(ThemeData theme) {
    return Row(
      children: [
        Text('Фотографии', style: theme.textTheme.titleSmall),
        const SizedBox(width: 8),
        Text(
          '${imageUrls.length}',
          style: theme.textTheme.bodySmall
              ?.copyWith(color: theme.colorScheme.outline),
        ),
        const Spacer(),
        Tooltip(
          message: 'Можно выбрать сразу несколько файлов',
          child: TextButton.icon(
            onPressed: busy ? null : onAdd,
            icon: const Icon(Icons.add_a_photo_outlined),
            label: const Text('Добавить'),
          ),
        ),
      ],
    );
  }

  Widget _progress(ThemeData theme, String text) {
    return Row(
      children: [
        SizedBox(
          width: 14,
          height: 14,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            color: theme.colorScheme.primary,
          ),
        ),
        const SizedBox(width: 8),
        Text(
          text,
          style: theme.textTheme.bodySmall
              ?.copyWith(color: theme.colorScheme.outline),
        ),
      ],
    );
  }
}

/// Миниатюра фотографии с кнопкой удаления поверх.
class PhotoTile extends StatelessWidget {
  const PhotoTile({
    super.key,
    required this.url,
    required this.busy,
    required this.onOpen,
    required this.onDelete,
  });

  final String url;
  final bool busy;
  final VoidCallback onOpen;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        InkWell(
          onTap: onOpen,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: CarImage(
              url: url,
              width: 104,
              height: 78,
              placeholderIcon: Icons.broken_image_outlined,
            ),
          ),
        ),
        Positioned(
          top: 2,
          right: 2,
          child: Material(
            color: Colors.black54,
            shape: const CircleBorder(),
            child: InkWell(
              customBorder: const CircleBorder(),
              onTap: busy ? null : onDelete,
              child: const Padding(
                padding: EdgeInsets.all(3),
                child: Icon(Icons.close, size: 16, color: Colors.white),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
