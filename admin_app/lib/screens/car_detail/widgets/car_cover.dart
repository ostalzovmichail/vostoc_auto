import 'package:flutter/material.dart';

import '../../../widgets/car_image.dart';

/// Обложка автомобиля (16:9) в верхней части карточки.
class CarCover extends StatelessWidget {
  const CarCover({super.key, required this.imageUrl});

  /// Полный адрес обложки; `null` — показываем плейсхолдер.
  final String? imageUrl;

  @override
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: 16 / 9,
      child: CarImage(
        url: imageUrl,
        placeholderIconSize: 64,
        placeholderIcon: Icons.directions_car,
      ),
    );
  }
}
