import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';

/// Выбранный пользователем файл с уже прочитанным содержимым.
typedef PickedFile = ({String name, Uint8List bytes});

/// Результат выбора фотографий.
class PickPhotosResult {
  const PickPhotosResult({required this.files, required this.errors});

  /// Успешно прочитанные файлы.
  final List<PickedFile> files;

  /// Сообщения об ошибках чтения (файл пустой/не читается).
  final List<String> errors;

  bool get isEmpty => files.isEmpty;
}

/// Открывает системный диалог выбора изображений и читает их в байты.
///
/// `file_picker` на всех платформах (включая web) отдаёт байты, поэтому путь
/// к файлу не используется — единый код для всех платформ.
Future<PickPhotosResult> pickPhotos() async {
  final files = await FilePicker.pickFiles(type: FileType.image);
  if (files.isEmpty) {
    return const PickPhotosResult(files: [], errors: []);
  }

  final picked = <PickedFile>[];
  final errors = <String>[];
  for (final file in files) {
    try {
      final bytes = await file.readAsBytes();
      if (bytes.isEmpty) {
        errors.add('Файл «${file.name}» пустой — пропущен');
        continue;
      }
      picked.add((name: file.name, bytes: bytes));
    } catch (error) {
      errors.add('Не удалось прочитать «${file.name}»: $error');
    }
  }
  return PickPhotosResult(files: picked, errors: errors);
}
