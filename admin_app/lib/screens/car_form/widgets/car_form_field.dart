import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Поле формы с валидацией.
///
/// Для числовых полей включает цифровую клавиатуру и фильтр `digitsOnly`.
class CarFormField extends StatelessWidget {
  const CarFormField({
    super.key,
    required this.controller,
    required this.label,
    required this.validator,
    this.hint,
    this.icon,
    this.number = false,
  });

  final TextEditingController controller;
  final String label;
  final String? Function(String?) validator;
  final String? hint;
  final IconData? icon;
  final bool number;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: TextFormField(
        controller: controller,
        keyboardType: number ? TextInputType.number : TextInputType.text,
        inputFormatters: number ? [FilteringTextInputFormatter.digitsOnly] : null,
        validator: validator,
        decoration: InputDecoration(
          labelText: label,
          hintText: hint,
          prefixIcon: icon == null ? null : Icon(icon),
          border: const OutlineInputBorder(),
        ),
      ),
    );
  }
}

/// Валидатор обязательного непустого поля.
String? requiredValidator(String? value) =>
    (value == null || value.trim().isEmpty) ? 'Обязательное поле' : null;

/// Валидатор целого числа с опциональными границами.
String? numberValidator(String? value, {int? min, int? max}) {
  final text = (value ?? '').trim();
  if (text.isEmpty) return 'Обязательное поле';
  final parsed = int.tryParse(text);
  if (parsed == null) return 'Введите число';
  if (min != null && parsed < min) return 'Не меньше $min';
  if (max != null && parsed > max) return 'Не больше $max';
  return null;
}
