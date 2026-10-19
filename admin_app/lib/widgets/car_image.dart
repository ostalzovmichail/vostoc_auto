import 'package:flutter/material.dart';

/// Сетевое изображение с плейсхолдером и обработкой ошибок загрузки.
///
/// Заменяет повторяющийся `Image.network` + `errorBuilder` в списке, обложке
/// и плитках фотографий — единый вид на всех экранах.
class CarImage extends StatelessWidget {
  const CarImage({
    super.key,
    required this.url,
    this.width,
    this.height,
    this.fit = BoxFit.cover,
    this.placeholderIcon = Icons.directions_car,
    this.placeholderIconSize,
  });

  /// Полный адрес изображения; `null`/пустой — показываем плейсхолдер.
  final String? url;
  final double? width;
  final double? height;
  final BoxFit fit;
  final IconData placeholderIcon;
  final double? placeholderIconSize;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final placeholder = _Placeholder(
      theme: theme,
      width: width,
      height: height,
      icon: placeholderIcon,
      iconSize: placeholderIconSize,
    );

    if (url == null || url!.isEmpty) return placeholder;

    return Image.network(
      url!,
      width: width,
      height: height,
      fit: fit,
      errorBuilder: (_, _, _) => _Placeholder(
        theme: theme,
        width: width,
        height: height,
        icon: Icons.broken_image_outlined,
        iconSize: placeholderIconSize,
      ),
    );
  }
}

class _Placeholder extends StatelessWidget {
  const _Placeholder({
    required this.theme,
    this.width,
    this.height,
    required this.icon,
    this.iconSize,
  });

  final ThemeData theme;
  final double? width;
  final double? height;
  final IconData icon;
  final double? iconSize;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: height,
      color: theme.colorScheme.surfaceContainerHighest,
      child: Icon(
        icon,
        size: iconSize,
        color: theme.colorScheme.outline,
      ),
    );
  }
}
