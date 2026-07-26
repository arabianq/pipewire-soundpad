# PipeWire Soundpad (PWSP) 🎵

<table border="0">
  <tr>
    <td colspan="2" align="center">
      <h3>Главная страница</h3>
      <img src="./pwsp-gui/assets/screenshots/main.png" alt="Main UI">
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <h3>Настройки</h3>
      <img src="./pwsp-gui/assets/screenshots/settings.png" alt="Settings">
    </td>
    <td width="50%" align="center">
      <h3>Горячие клавиши</h3>
      <img src="./pwsp-gui/assets/screenshots/hotkeys.png" alt="Hotkeys">
    </td>
  </tr>
</table>

🇷🇺 Русский (вы здесь) | [🇬🇧 Read in English](README.md)

[![GitHub Actions Build Status](https://img.shields.io/github/actions/workflow/status/arabianq/pipewire-soundpad/build.yml?branch=main&style=flat-square)](https://github.com/arabianq/pipewire-soundpad/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![GitHub Release](https://img.shields.io/github/v/release/arabianq/pipewire-soundpad?style=flat-square)](https://github.com/arabianq/pipewire-soundpad/releases/latest)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20PipeWire-blue?style=flat-square)](https://pipewire.org/)

**PipeWire Soundpad (PWSP)** — приложение, которое позволяет воспроизводить аудиофайлы прямо в ваш микрофон. Разработано специально для Linux и работает с PipeWire напрямую: создаёт собственный виртуальный микрофон и само выстраивает маршрутизацию, так что настраивать её руками не придётся.

---

## ✨ Возможности

- **Нативная интеграция с PipeWire:** PWSP создаёт собственный виртуальный микрофон и управляет графом через PipeWire API — без ручного `pw-link` и без подгрузки модулей PulseAudio.
- **Раздельная громкость мониторинга и микрофона:** Два независимых потока: можно усилить то, что слышат собеседники, не оглушив себя, и выбрать устройство для мониторинга.
- **Модульная архитектура:** Состоит из фонового демона (`daemon`), интерфейса командной строки (`cli`) и графического интерфейса (`gui`).
- **Современный GUI:** Построен на `egui`, плавно работает как на Wayland, так и на X11.
- **Локализация:** 9 языков; используется системный или выбранный вручную в настройках.
- **Глобальные горячие клавиши:** Работают через `evdev`, позволяя воспроизводить звуки из любого окна.
- **Широкая поддержка форматов:** Использует `rodio` и `symphonia` и поддерживает большинство аудиоформатов, включая Opus.

---

## 🚀 Установка

Мы предоставляем несколько способов установки PWSP, включая стабильные релизы и постоянно обновляемые "nightly" сборки напрямую из ветки `main`.

### 📦 Flatpak (Рекомендуется)

Добавьте наш официальный OSTree репозиторий и установите приложение:

```bash
# Добавьте репозиторий
flatpak remote-add --if-not-exists pwsp https://arabianq.github.io/pipewire-soundpad/pwsp.flatpakrepo

# Установите стабильную (Stable) версию
flatpak install pwsp ru.arabianq.pwsp//stable

# ИЛИ установите Nightly версию (самые свежие обновления)
flatpak install pwsp ru.arabianq.pwsp//nightly
```

### 🟠 Debian / Ubuntu (APT Репозиторий)

Мы поддерживаем официальный APT-репозиторий для бесшовных обновлений через `apt`:

```bash
# 1. Скачайте публичный GPG ключ
sudo mkdir -p /etc/apt/keyrings
wget -O- https://arabianq.github.io/pipewire-soundpad/apt/pubkey.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/pwsp.gpg

# 2. Добавьте репозиторий (Выберите STABLE или NIGHTLY)
# Для Stable:
echo "deb [signed-by=/etc/apt/keyrings/pwsp.gpg] https://arabianq.github.io/pipewire-soundpad/apt/ stable main" | sudo tee /etc/apt/sources.list.d/pwsp.list

# Для Nightly:
# echo "deb [signed-by=/etc/apt/keyrings/pwsp.gpg] https://arabianq.github.io/pipewire-soundpad/apt/ nightly main" | sudo tee /etc/apt/sources.list.d/pwsp.list

# 3. Обновите индексы и установите
sudo apt update
sudo apt install pwsp
```

### 🐧 Fedora / RHEL (COPR)

Доступно через репозиторий Fedora COPR:

```bash
sudo dnf copr enable arabianq/pwsp
sudo dnf install pwsp
```

### 🐦 Arch Linux (AUR)

Доступны два пакета: `pwsp` собирается из исходников, `pwsp-bin` ставит готовые бинарники.

```bash
# Через любой удобный AUR-хелпер
paru -S pwsp      # или: pwsp-bin
```

### ⚙️ Ручная установка

Вы можете вручную скачать пакеты `.deb` или готовые бинарники `.zip` на [странице релизов](https://github.com/arabianq/pipewire-soundpad/releases).

### 🦀 Сборка из исходников

Убедитесь, что у вас установлены Rust и Cargo, а также зависимости для сборки. Для Debian/Ubuntu:

```bash
sudo apt install libpipewire-0.3-dev libclang-dev libasound2-dev libdbus-1-dev libssl-dev pkg-config
```

```bash
git clone https://github.com/arabianq/pipewire-soundpad.git
cd pipewire-soundpad
cargo build --release --locked
```

Собранные бинарники будут находиться в папке `target/release/`.

---

## 🎮 Использование

### 1. Запуск демона

PWSP работает через фоновый демон, который маршрутизирует аудио.

```bash
# Запуск демона
pwsp-daemon
```

_(Подсказка: Если вы установили программу через пакетный менеджер, она включает пользовательский systemd-сервис. Вы можете включить его командой: `systemctl --user enable --now pwsp-daemon.service`)_

### 2. Запуск GUI

Просто запустите графический интерфейс для управления и воспроизведения звуков:

```bash
pwsp-gui
```

### 3. Использование CLI

Вы также можете взаимодействовать с демоном напрямую через командную строку. Все команды сгруппированы в `action`, `get` и `set`:

```bash
pwsp-cli action play /path/to/sound.mp3
pwsp-cli action stop
pwsp-cli get state

# Раздельная громкость: что слышите вы и что уходит в микрофон.
# Значения выше 1.0 усиливают.
pwsp-cli set monitoring-volume 0.3
pwsp-cli set mic-volume 1.5

# Устройства
pwsp-cli get inputs
pwsp-cli set input <имя>
```

Полный список — `pwsp-cli <группа> --help`.

### 🔑 Включение глобальных горячих клавиш

Горячие клавиши читаются напрямую из ядра через `evdev`, поэтому демону нужен доступ к `/dev/input/event*`. В большинстве дистрибутивов для этого достаточно добавить себя в группу `input`:

```bash
sudo usermod -aG input $USER
```

Изменение вступит в силу после перезахода в систему. Без него всё остальное работает — молчат только глобальные горячие клавиши.

### 📦 Замечание для пользователей Flatpak

У Flatpak-версии одна точка входа, поэтому команды выше вызываются через `flatpak run`:

```bash
# Без аргументов: запускает демон и GUI вместе
flatpak run ru.arabianq.pwsp

# Отдельно демон
flatpak run ru.arabianq.pwsp daemon --start
flatpak run ru.arabianq.pwsp daemon --kill

# Всё после "cli" передаётся в pwsp-cli
flatpak run ru.arabianq.pwsp cli action play /path/to/sound.mp3
```

---

## 📚 Документация и DeepWiki

DeepWiki автоматически строит по исходникам обзор архитектуры и позволяет задавать по нему вопросы. Он генерируется, а не пишется руками, поэтому воспринимайте его как карту, а не как спецификацию:

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/arabianq/pipewire-soundpad)

---

## 🤝 Вклад в проект (Contributing)

Будем рады вашей помощи, баг-репортам и идеям!

1. Сделайте Fork проекта.
2. Создайте свою ветку (`git checkout -b feat/amazing-feature`).
3. Закоммитьте изменения (`git commit -m 'Add some amazing feature'`).
4. Запушьте ветку (`git push origin feat/amazing-feature`).
5. Откройте Pull Request.

---

## 📝 Лицензия

Распространяется под лицензией MIT. Подробнее см. в файле `LICENSE`.

_Сделано с ❤️ для сообщества Linux._
