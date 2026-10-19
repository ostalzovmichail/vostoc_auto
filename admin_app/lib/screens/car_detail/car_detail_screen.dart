import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../models/car.dart';
import '../../services/api_client.dart';
import '../../services/photo_picker.dart';
import '../../utils/dialogs.dart';
import '../../utils/ui_feedback.dart';
import '../../constants/text_constants.dart';
import 'photo_viewer_screen.dart';
import 'widgets/car_cover.dart';
import 'widgets/photos_section.dart';
import 'widgets/price_card.dart';
import 'widgets/specs_card.dart';

/// Карточка автомобиля: характеристики, смена статуса и управление фото.
class CarDetailScreen extends ConsumerStatefulWidget {
  const CarDetailScreen({super.key, required this.car});

  final Car car;

  @override
  ConsumerState<CarDetailScreen> createState() => _CarDetailScreenState();
}

class _CarDetailScreenState extends ConsumerState<CarDetailScreen> {
  late Car _car;
  bool _busy = false;

  /// Текст прогресса при загрузке нескольких фото, `null` — ничего не грузится.
  String? _uploadStatus;

  @override
  void initState() {
    super.initState();
    _car = widget.car;
  }

  /// Показать индикатор, выполнить операцию, обновить карточку.
  Future<void> _run(Future<Car> Function(ApiClient api) action) async {
    setState(() => _busy = true);
    try {
      final updated = await action(ref.read(apiClientProvider));
      if (!mounted) return;
      setState(() => _car = updated);
    } on ApiException catch (error) {
      if (mounted) showMessage(context, error.message, isError: true);
    } catch (error) {
      if (mounted) showMessage(context, '${TextConstants.errorPrefix}$error', isError: true);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _changeStatus(CarStatus status) async {
    if (status == _car.statusEnum) return;

    // Для «продано» уточняем город доставки — он показывается в каталоге.
    String? city;
    if (status == CarStatus.sold) {
      city = await askTextDialog(
        context,
        title: DetailTexts.deliveryCityTitle,
        hint: DetailTexts.deliveryCityHint,
        confirmLabel: DetailTexts.deliveryCityConfirm,
      );
      if (city == null) return; // отмена
    }

    await _run((api) => api.setStatus(_car.id, status, deliveryCity: city));
    if (mounted) showMessage(context, 'Статус: ${status.label}');
  }

  Future<void> _addPhotos() async {
    final result = await pickPhotos();
    for (final error in result.errors) {
      if (mounted) showMessage(context, error, isError: true);
    }
    if (result.isEmpty || !mounted) return;

    final total = result.files.length;
    setState(() => _busy = true);
    try {
      final api = ref.read(apiClientProvider);
      var latest = _car;
      for (var i = 0; i < total; i++) {
        setState(() => _uploadStatus = DetailTexts.uploadProgress.replaceFirst('%d', (i + 1).toString()).replaceFirst('%d', total.toString()));
        latest = await api.uploadImage(
          _car.id,
          fileName: result.files[i].name,
          bytes: result.files[i].bytes,
        );
      }
      if (!mounted) return;
      setState(() {
        _car = latest;
        _uploadStatus = null;
      });
      showMessage(context, total == 1 ? 'Фото добавлено' : 'Добавлено фото: $total');
    } on ApiException catch (error) {
      if (mounted) showMessage(context, error.message, isError: true);
    } catch (error) {
      if (mounted) showMessage(context, 'Ошибка: $error', isError: true);
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
          _uploadStatus = null;
        });
      }
    }
  }

  Future<void> _deletePhoto(String url) async {
    await _run((api) => api.deleteImage(_car.id, url));
    if (mounted) showMessage(context, 'Фото удалено');
  }

  Future<void> _openPhoto(String url) async {
    final deleteRequested = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => PhotoViewerScreen(
          imageUrl: ref.read(apiClientProvider).imageUrl(url) ?? '',
        ),
      ),
    );
    if (deleteRequested == true) await _deletePhoto(url);
  }

  Future<void> _deleteCar() async {
    final confirmed = await confirmDialog(
      context,
      title: 'Удалить автомобиль?',
      message: '${_car.title} (лот ${_car.lotNumber}) и все его фотографии будут '
          'удалены безвозвратно.',
      action: 'Удалить',
      destructive: true,
    );
    if (!confirmed) return;

    try {
      await ref.read(apiClientProvider).deleteCar(_car.id);
      if (!mounted) return;
      Navigator.of(context).pop(true);
    } on ApiException catch (error) {
      if (mounted) showMessage(context, error.message, isError: true);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final api = ref.read(apiClientProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(_car.title),
        actions: [
          if (_busy)
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            ),
          IconButton(
            tooltip: 'Удалить автомобиль',
            icon: const Icon(Icons.delete_outline),
            onPressed: _deleteCar,
          ),
        ],
      ),
      body: ListView(
        children: [
          CarCover(imageUrl: api.imageUrl(_car.coverPath)),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Статус', style: theme.textTheme.titleSmall),
                const SizedBox(height: 8),
                _statusChips(),
                const SizedBox(height: 20),
                PriceCard(car: _car),
                const SizedBox(height: 16),
                SpecsCard(car: _car),
                const SizedBox(height: 20),
                PhotosSection(
                  imageUrls: [
                    for (final url in _car.images) api.imageUrl(url) ?? '',
                  ],
                  busy: _busy,
                  uploadStatus: _uploadStatus,
                  onAdd: _addPhotos,
                  onOpen: _openPhoto,
                  onDelete: _deletePhoto,
                ),
              ],
            ),
          ),
        ],
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
              selected: _car.statusEnum == status,
              onSelected: (_) => _changeStatus(status),
            ),
          )
          .toList(),
    );
  }
}
