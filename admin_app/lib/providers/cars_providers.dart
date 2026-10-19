import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../models/car.dart';
import '../services/api_client.dart';

part 'cars_providers.g.dart';

/// Фильтр статуса для каталога: `null` — все автомобили.
@riverpod
class CarStatusFilter extends _$CarStatusFilter {
  @override
  CarStatus? build() => null;

  void set(CarStatus? status) => state = status;
}

/// Список автомобилей с учётом текущего фильтра.
@riverpod
Future<List<Car>> carsList(Ref ref) async {
  final filter = ref.watch(carStatusFilterProvider);
  final api = ref.watch(apiClientProvider);
  return api.listCars(status: filter?.value);
}
