# AK500 DIGITAL Lite — инструкция

Это прототип на Rust для дисплея DeepCool AK500 DIGITAL WH под Windows 10/11.

**Работает:** вывод загрузки процессора по USB HID без PawnIO и официального DeepCool.

**Не реализовано:** чтение реальной температуры Ryzen 7 7800X3D. Для температуры требуется проверенный аппаратный интерфейс, обычный API Windows её не предоставляет.

## Сборка

Установи Rust с https://rustup.rs/ и Windows MSVC Build Tools. Открой PowerShell в папке проекта:

```powershell
cargo build --release
cargo test
```

Бинарник: `target\release\ak500-digital-lite.exe`.

## Безопасный тест

```powershell
.\target\release\ak500-digital-lite.exe --dry-run --once
```

Далее проверка USB:

```powershell
.\target\release\ak500-digital-lite.exe --list
```

Ожидаются VID `3633` и PID `0003`. Если отличаются — сначала выясни реальный идентификатор, не изменяй PID наугад.

После закрытия оригинального приложения DeepCool (его службы также могут занимать HID-интерфейс):

```powershell
.\target\release\ak500-digital-lite.exe --once
.\target\release\ak500-digital-lite.exe --quiet
```

Остановка: Ctrl+C. Автозапуск не меняется.

Тестовая температура (фиксированное значение, НЕ реальное измерение):

```powershell
.\target\release\ak500-digital-lite.exe --test-temp 45
```

Держится 2 секунды, затем заменяется одним измерением загрузки CPU.

## Ограничения

- Исходники пока не протестированы на реальном AK500 DIGITAL WH и не проверены сборкой под Windows.
- Нет управления вентиляторами, BIOS, службами Windows, драйверами или реестром.
- Без достоверного интерфейса датчика настоящую температуру CPU Tdie получить нельзя.
- Проверять потребление памяти и размер конечного .exe следует после сборки на компьютере.
