// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'cars_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Фильтр статуса для каталога: `null` — все автомобили.

@ProviderFor(CarStatusFilter)
final carStatusFilterProvider = CarStatusFilterProvider._();

/// Фильтр статуса для каталога: `null` — все автомобили.
final class CarStatusFilterProvider
    extends $NotifierProvider<CarStatusFilter, CarStatus?> {
  /// Фильтр статуса для каталога: `null` — все автомобили.
  CarStatusFilterProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'carStatusFilterProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$carStatusFilterHash();

  @$internal
  @override
  CarStatusFilter create() => CarStatusFilter();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(CarStatus? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<CarStatus?>(value),
    );
  }
}

String _$carStatusFilterHash() => r'4be8d33e8cc5a32ee154f793ea527713583a0fc4';

/// Фильтр статуса для каталога: `null` — все автомобили.

abstract class _$CarStatusFilter extends $Notifier<CarStatus?> {
  CarStatus? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<CarStatus?, CarStatus?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<CarStatus?, CarStatus?>,
              CarStatus?,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}

/// Список автомобилей с учётом текущего фильтра.

@ProviderFor(carsList)
final carsListProvider = CarsListProvider._();

/// Список автомобилей с учётом текущего фильтра.

final class CarsListProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<Car>>,
          List<Car>,
          FutureOr<List<Car>>
        >
    with $FutureModifier<List<Car>>, $FutureProvider<List<Car>> {
  /// Список автомобилей с учётом текущего фильтра.
  CarsListProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'carsListProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$carsListHash();

  @$internal
  @override
  $FutureProviderElement<List<Car>> $createElement($ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Car>> create(Ref ref) {
    return carsList(ref);
  }
}

String _$carsListHash() => r'0a6136d268b9fc7e44765bc1fe932f459a390013';
