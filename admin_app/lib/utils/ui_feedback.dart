import 'package:flutter/material.dart';

/// Показывает сообщение в SnackBar.
///
/// Единая точка для всех уведомлений: ошибки подсвечиваются цветом
/// `colorScheme.error`, обычные сообщения используют тему по умолчанию.
void showMessage(
  BuildContext context,
  String text, {
  bool isError = false,
}) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(
      content: Text(text),
      backgroundColor: isError ? Theme.of(context).colorScheme.error : null,
    ),
  );
}
