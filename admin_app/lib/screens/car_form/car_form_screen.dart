import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../models/car.dart';
import '../../services/api_client.dart';
import '../../constants/text_constants.dart';
import 'widgets/car_form_field.dart';

/// Форма добавления автомобиля в каталог.
///
/// Возвращает `true` через `Navigator.pop`, если запись создана — тогда
/// родительский экран обновляет список.
class CarFormScreen extends ConsumerStatefulWidget {
  const CarFormScreen({super.key});

  @override
  ConsumerState<CarFormScreen> createState() => _CarFormScreenState();
}

class _CarFormScreenState extends ConsumerState<CarFormScreen> {
  final _formKey = GlobalKey<FormState>();

  final _brand = TextEditingController();
  final _model = TextEditingController();
  final _year = TextEditingController();
  final _mileage = TextEditingController();
  final _engine = TextEditingController();
  final _transmission = TextEditingController();
  final _drive = TextEditingController();
  final _price = TextEditingController();
  final _lot = TextEditingController();

  CarStatus _status = CarStatus.stock;
  bool _saving = false;

  @override
  void dispose() {
    for (final controller in [
      _brand,
      _model,
      _year,
      _mileage,
      _engine,
      _transmission,
      _drive,
      _price,
      _lot,
    ]) {
      controller.dispose();
    }
    super.dispose();
  }

  Future<void> _submit() async {
    if (!(_formKey.currentState?.validate() ?? false)) return;

    setState(() => _saving = true);
    final api = ref.read(apiClientProvider);

    try {
      await api.createCar(
        CarDraft(
          brand: _brand.text,
          model: _model.text,
          year: int.parse(_year.text.trim()),
          mileage: int.parse(_mileage.text.trim()),
          engine: _engine.text,
          transmission: _transmission.text,
          drive: _drive.text,
          priceRub: int.parse(_price.text.trim()),
          lotNumber: _lot.text,
          status: _status,
        ),
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text(FormTexts.addedMessage)),
      );
      Navigator.of(context).pop(true);
    } on ApiException catch (error) {
      _showError(error.message);
    } catch (error) {
      _showError('${FormTexts.saveError}$error');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  void _showError(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: Theme.of(context).colorScheme.error,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text(FormTexts.appBarTitle)),
      body: Form(
        key: _formKey,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            CarFormField(
              controller: _brand,
              label: FormTexts.brandLabel,
              hint: FormTexts.brandHint,
              icon: Icons.branding_watermark_outlined,
              validator: requiredValidator,
            ),
            CarFormField(
              controller: _model,
              label: FormTexts.modelLabel,
              hint: FormTexts.modelHint,
              icon: Icons.directions_car_outlined,
              validator: requiredValidator,
            ),
            Row(
              children: [
                Expanded(
                  child: CarFormField(
                    controller: _year,
                    label: FormTexts.yearLabel,
                    hint: FormTexts.yearHint,
                    icon: Icons.event_outlined,
                    number: true,
                    validator: (v) => numberValidator(v, min: 1950, max: 2100),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: CarFormField(
                    controller: _mileage,
                    label: FormTexts.mileageLabel,
                    hint: FormTexts.mileageHint,
                    icon: Icons.speed_outlined,
                    number: true,
                    validator: (v) => numberValidator(v, min: 0),
                  ),
                ),
              ],
            ),
            CarFormField(
              controller: _engine,
              label: FormTexts.engineLabel,
              hint: FormTexts.engineHint,
              icon: Icons.settings_suggest_outlined,
              validator: requiredValidator,
            ),
            CarFormField(
              controller: _transmission,
              label: FormTexts.transmissionLabel,
              hint: FormTexts.transmissionHint,
              icon: Icons.swap_horiz,
              validator: requiredValidator,
            ),
            CarFormField(
              controller: _drive,
              label: FormTexts.driveLabel,
              hint: FormTexts.driveHint,
              icon: Icons.sync_alt,
              validator: requiredValidator,
            ),
            CarFormField(
              controller: _price,
              label: FormTexts.priceLabel,
              hint: FormTexts.priceHint,
              icon: Icons.payments_outlined,
              number: true,
              validator: (v) => numberValidator(v, min: 1),
            ),
            CarFormField(
              controller: _lot,
              label: FormTexts.lotLabel,
              hint: FormTexts.lotHint,
              icon: Icons.confirmation_number_outlined,
              validator: (_) => null,
            ),
            const SizedBox(height: 8),
            Text(FormTexts.statusLabel, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 8),
            _statusChips(),
            const SizedBox(height: 24),
            _submitButton(),
          ],
        ),
      ),
    );
  }

  Widget _statusChips() {
    return Wrap(
      spacing: 8,
      children: CarStatus.values
          .map(
            (status) => ChoiceChip(
              label: Text(status.label),
              selected: _status == status,
              onSelected: (_) => setState(() => _status = status),
            ),
          )
          .toList(),
    );
  }

  Widget _submitButton() {
    return FilledButton.icon(
      onPressed: _saving ? null : _submit,
      icon: _saving
          ? const SizedBox(
              width: 18,
              height: 18,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.save_outlined),
      label: const Text(TextConstants.save),
    );
  }
}
