# Test Report - ВОСТОК АВТО · Управление

**Date:** 2025-10-19
**Flutter Version:** 3.13.5 (from environment)
**Project:** bereg_admin

---

## Unit Tests Results ✅

**Command:** `flutter test`

```
00:00 +0: loading /Users/michail/progect/bereg/admin_app/test/car_model_test.dart
00:00 +0: Car.fromJson читает полный ответ API
00:00 +1: Car.fromJson не падает на отсутствующих необязательных полях
00:00 +2: Car.fromJson неизвестный статус считается «в наличии»
00:00 +3: форматирование цена разделяется по тысячам
00:00 +4: форматирование пробег разделяется по тысячам
00:00 +5: All tests passed!
```

**Summary:** 5 tests passed, 0 failed.

### Test Coverage (test/car_model_test.dart)
| Test Group | Test Case | Status |
|------------|-----------|--------|
| Car.fromJson | читает полный ответ API | ✅ PASSED |
| Car.fromJson | не падает на отсутствующих необязательных полях | ✅ PASSED |
| Car.fromJson | неизвестный статус считается «в наличии» | ✅ PASSED |
| форматирование | цена разделяется по тысячам | ✅ PASSED |
| форматирование | пробег разделяется по тысячам | ✅ PASSED |

---

## Static Analysis Results ❌

**Command:** `flutter analyze`

**Total Issues:** 48 (41 errors, 2 warnings, 5 info)

### Critical Errors (prevent compilation/code generation)

#### 1. Syntax Errors
| File | Line | Issue |
|------|------|-------|
| `lib/screens/car_detail_screen.dart` | 7:8 | Missing semicolon after import `../models/car.dart` |
| `lib/screens/settings_screen.dart` | 164:5 | Missing semicolon (expected token) |

#### 2. Missing Generated Files (Riverpod code generation)
| File | Line | Issue |
|------|------|-------|
| `lib/services/api_client.dart` | 9:6 | Target URI `api_client.g.dart` not generated |
| `lib/services/settings.dart` | 5:6 | Target URI `settings.g.dart` not generated |

#### 3. Undefined Providers/Classes (due to missing code generation)
| File | Issue |
|------|-------|
| `lib/main.dart` | `appSettingsProvider`, `AppSettingsState.loading`, `AppSettingsState.loaded` undefined |
| `lib/screens/home_screen.dart` | `riverpod` annotation, `CarsListRef`, `carStatusFilterProvider`, `apiClientProvider`, `carsListProvider`, `state` undefined |
| `lib/screens/car_detail_screen.dart` | `apiClientProvider` undefined (6 occurrences) |
| `lib/screens/car_form_screen.dart` | `apiClientProvider` undefined |
| `lib/screens/settings_screen.dart` | `appSettingsProvider` undefined (2 occurrences) |
| `lib/services/api_client.dart` | `ApiClientRef`, `appSettingsProvider`, `loaded` undefined |
| `lib/services/settings.dart` | `state` undefined (3 occurrences), object pattern issues |

#### 4. Code Pattern Issues (Dart version compatibility)
| File | Line | Issue |
|------|------|-------|
| `lib/services/settings.dart` | 86:33 | Positional field in object pattern (requires Dart 3.0+) |
| `lib/services/settings.dart` | 93:15 | Positional field in object pattern (requires Dart 3.0+) |
| `lib/services/settings.dart` | 106:16 | Missing `@override` on `baseUrl` |
| `lib/services/settings.dart` | 107:16 | Missing `@override` on `token` |

#### 5. Warnings
| File | Line | Issue |
|------|------|-------|
| `lib/screens/home_screen.dart` | 15:14 | `@override` on non-overriding member |
| `lib/services/settings.dart` | 28:20 | `@override` on non-overriding member |

#### 6. Info (Best Practices)
| File | Line | Issue |
|------|------|-------|
| `lib/screens/home_screen.dart` | 389:30 | BuildContext across async gaps |
| `lib/screens/home_screen.dart` | 407:30 | BuildContext across async gaps |

---

## Root Cause Analysis

The majority of static analysis errors (39 of 41 errors) are **cascading failures** from two root causes:

1. **Syntax errors** in `car_detail_screen.dart` and `settings_screen.dart` prevent the Riverpod generator from running
2. **Missing generated files** (`*.g.dart`) cause all provider references to be undefined

Once the two syntax errors are fixed and `flutter pub run build_runner build --delete-conflicting-outputs` is run, the generated files will be created and most errors should resolve.

---

## Recommendations

1. **Fix syntax errors** (blocking code generation):
   - Add semicolon after `import '../models/car.dart'` in `car_detail_screen.dart:7`
   - Fix missing semicolon in `settings_screen.dart:164`

2. **Run code generation:**
   ```bash
   flutter pub run build_runner build --delete-conflicting-outputs
   ```

3. **Re-run analysis** after generation to verify remaining issues

4. **Consider Dart version:** The project uses SDK `^3.13.5` but analyzer is on 3.9.0. Run `flutter packages upgrade` to update analyzer.

---

## Next Steps

If fixes are provided in this `test_report/` folder, re-run:
1. `flutter test`
2. `flutter analyze`
3. Update this report with new results