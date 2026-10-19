import 'package:flutter/material.dart';

import '../../../widgets/car_image.dart';

/// Полноэкранный просмотр фотографии с возможностью удалить её.
///
/// Возвращает `true` через `Navigator.pop`, если пользователь запросил
/// удаление — решение об удалении принимает вызывающий экран.
class PhotoViewerScreen extends StatelessWidget {
  const PhotoViewerScreen({super.key, required this.imageUrl});

  final String imageUrl;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        actions: [
          IconButton(
            tooltip: 'Удалить фото',
            icon: const Icon(Icons.delete_outline),
            onPressed: () => Navigator.of(context).pop(true),
          ),
        ],
      ),
      body: Center(
        child: InteractiveViewer(
          child: CarImage(url: imageUrl),
        ),
      ),
    );
  }
}
