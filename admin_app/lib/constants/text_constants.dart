/// Текстовые константы приложения.
class TextConstants {
  TextConstants._();

  /// Общие сообщения и UI-элементы.
  static const String errorPrefix = 'Ошибка: ';
  static const String statusPrefix = 'Статус: ';
  static const String save = 'Сохранить';
  static const String add = 'Добавить';
  static const String settings = 'Настройки';
}

/// Экран каталога (Home).
class HomeTexts {
  HomeTexts._();

  static const String appBarTitle = 'ВОСТОК АВТО';
  static const String emptyTitle = 'Список пуст';
  static const String emptySubtitle = 'Добавьте первый автомобиль кнопкой «Добавить».';
  static const String loadErrorTitle = 'Не удалось загрузить каталог';
  static const String markSoldTitle = 'Отметить проданным?';
  static const String markSoldAction = 'Продано';
  static const String deleteTitle = 'Удалить автомобиль?';
  static const String deleteAction = 'Удалить';
  static const String deletedMessage = 'Автомобиль удалён';
}

/// Экран настроек (Settings).
class SettingsTexts {
  SettingsTexts._();

  static const String firstRunTitle = 'Подключение';
  static const String normalTitle = 'Настройки';
  static const String serverTitle = 'Сервер автосалона';
  static const String serverDescription =
      'Приложение управляет каталогом через API сайта: адрес сервера и токен администратора (заголовок X-Admin-Token).';
  static const String baseUrlLabel = 'Адрес сервера';
  static const String baseUrlHint = 'http://127.0.0.1:8080';
  static const String tokenLabel = 'Токен администратора';
  static const String tokenShow = 'Показать';
  static const String tokenHide = 'Скрыть';
  static const String testConnection = 'Проверить связь';
  static const String enter = 'Войти';
  static const String savedMessage = 'Настройки сохранены';
  static const String connectionSuccess = 'Связь есть: сервер вернул ';
  static const String connectionError = 'Не удалось подключиться: ';
}

/// Экран деталей (Car Detail).
class DetailTexts {
  DetailTexts._();

  static const String deliveryCityTitle = 'Город доставки';
  static const String deliveryCityHint = 'Например, Москва';
  static const String deliveryCityConfirm = 'Продано';
  static const String uploadProgress = 'Загружаю фото %d из %d…';
}

/// Диалог со счётчиком посещений сайта.
class VisitsTexts {
  VisitsTexts._();

  static const String tooltip = 'Посещения сайта';
  static const String title = 'Посещения сайта';
  static const String total = 'Всего';
  static const String today = 'Сегодня';
  static const String last7Days = 'За 7 дней';
  static const String refresh = 'Обновить';
  static const String close = 'Закрыть';
  static const String loadError = 'Не удалось загрузить: ';
}

/// Экран формы (Car Form).
class FormTexts {
  FormTexts._();

  static const String appBarTitle = 'Новый автомобиль';
  static const String addedMessage = 'Автомобиль добавлен';
  static const String saveError = 'Не удалось сохранить: ';
  static const String brandLabel = 'Марка';
  static const String brandHint = 'Audi';
  static const String modelLabel = 'Модель';
  static const String modelHint = 'A3';
  static const String yearLabel = 'Год';
  static const String yearHint = '2021';
  static const String mileageLabel = 'Пробег, км';
  static const String mileageHint = '55200';
  static const String engineLabel = 'Двигатель';
  static const String engineHint = '1.4T (150 л.с.)';
  static const String transmissionLabel = 'Коробка передач';
  static const String transmissionHint = 'S tronic';
  static const String driveLabel = 'Привод';
  static const String driveHint = '2WD';
  static const String priceLabel = 'Цена, ₽';
  static const String priceHint = '1940000';
  static const String lotLabel = 'Номер лота (необязательно)';
  static const String lotHint = 'оставьте пустым — присвоится автоматически';
  static const String statusLabel = 'Статус';
}
