import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../models/car.dart';
import '../../providers/cars_providers.dart';
import '../../services/api_client.dart';
import '../../utils/dialogs.dart';
import '../../utils/ui_feedback.dart';
import '../../constants/text_constants.dart';
import '../../widgets/message_view.dart';
import '../car_detail/car_detail_screen.dart';
import '../car_form/car_form_screen.dart';
import '../settings/settings_screen.dart';
import 'widgets/car_card.dart';
import 'widgets/status_filter_bar.dart';
import 'widgets/visits_dialog.dart';

/// Каталог автомобилей: список, фильтр по статусу, быстрые действия.
class HomeScreen extends ConsumerStatefulWidget {
  const HomeScreen({super.key});

  @override
  ConsumerState<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends ConsumerState<HomeScreen> {
  @override
  Widget build(BuildContext context) {
    final carsAsync = ref.watch(carsListProvider);
    final filter = ref.watch(carStatusFilterProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text(HomeTexts.appBarTitle),
        actions: [
          carsAsync.maybeWhen(
            loading: () => const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            ),
            orElse: () => const SizedBox.shrink(),
          ),
          IconButton(
            tooltip: VisitsTexts.tooltip,
            icon: const Icon(Icons.insights_outlined),
            onPressed: _openVisits,
          ),
          IconButton(
            tooltip: TextConstants.settings,
            icon: const Icon(Icons.settings_outlined),
            onPressed: _openSettings,
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _openForm,
        icon: const Icon(Icons.add),
        label: const Text(TextConstants.add),
      ),
      body: Column(
        children: [
          StatusFilterBar(selected: filter),
          Expanded(
            child: RefreshIndicator(
              onRefresh: () async {
                ref.invalidate(carsListProvider);
                await ref.read(carsListProvider.future);
              },
              child: carsAsync.when(
                data: (cars) => cars.isEmpty
                    ? const MessageView(
                        title: HomeTexts.emptyTitle,
                        subtitle: HomeTexts.emptySubtitle,
                        icon: Icons.directions_car_outlined,
                      )
                    : _carList(cars),
                loading: () => const Center(child: CircularProgressIndicator()),
                error: (error, _) => MessageView(
                  title: HomeTexts.loadErrorTitle,
                  subtitle: error.toString(),
                  icon: Icons.cloud_off_outlined,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _carList(List<Car> cars) {
    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(12, 4, 12, 96),
      itemCount: cars.length,
      itemBuilder: (context, index) => CarCard(
        car: cars[index],
        onOpen: _openDetail,
        onMarkSold: _markSold,
        onDelete: _delete,
      ),
    );
  }

  Future<void> _openForm() async {
    final created = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const CarFormScreen()),
    );
    if (created == true) ref.invalidate(carsListProvider);
  }

  Future<void> _openDetail(Car car) async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => CarDetailScreen(car: car)),
    );
    // Детали могли изменить статус или фотографии — список перечитываем.
    ref.invalidate(carsListProvider);
  }

  Future<void> _openSettings() async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const SettingsScreen()),
    );
    // Настройки могли поменяться — apiClient пересоберётся сам через watch.
    ref.invalidate(carsListProvider);
  }

  /// Открывает счётчик посещений сайта — только для администратора.
  void _openVisits() {
    showDialog<void>(
      context: context,
      builder: (_) => const VisitsDialog(),
    );
  }

  /// Выполняет операцию и обновляет список, показывая ошибку в SnackBar.
  Future<void> _run(Future<void> Function(ApiClient api) action) async {
    try {
      await action(ref.read(apiClientProvider));
      ref.invalidate(carsListProvider);
    } on ApiException catch (error) {
      if (mounted) showMessage(context, error.message, isError: true);
    } catch (error) {
      if (mounted) showMessage(context, '${TextConstants.errorPrefix}$error', isError: true);
    }
  }

  Future<void> _markSold(Car car) async {
    final confirmed = await confirmDialog(
      context,
      title: HomeTexts.markSoldTitle,
      message: '${car.title} (лот ${car.lotNumber}) перейдёт в статус «Продано».',
      action: HomeTexts.markSoldAction,
    );
    if (!confirmed) return;

    await _run((api) => api.setStatus(car.id, CarStatus.sold));
    if (mounted) showMessage(context, '${car.title}: статус «Продано»');
  }

  Future<void> _delete(Car car) async {
    final confirmed = await confirmDialog(
      context,
      title: HomeTexts.deleteTitle,
      message: '${car.title} (лот ${car.lotNumber}) и все его фотографии будут '
          'удалены безвозвратно.',
      action: HomeTexts.deleteAction,
      destructive: true,
    );
    if (!confirmed) return;

    await _run((api) => api.deleteCar(car.id));
    if (mounted) showMessage(context, HomeTexts.deletedMessage);
  }
}
