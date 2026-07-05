# PipeWire Soundpad (PWSP) 🎵

![PWSP Screenshot](https://raw.githubusercontent.com/arabianq/pipewire-soundpad/master/assets/screenshot.png)

[🇷🇺 Читать на русском](README.ru.md) | [🇺🇸 Read in English](README.md)

[![GitHub Actions Build Status](https://img.shields.io/github/actions/workflow/status/arabianq/pipewire-soundpad/build.yml?branch=main&style=flat-square)](https://github.com/arabianq/pipewire-soundpad/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![GitHub Release](https://img.shields.io/github/v/release/arabianq/pipewire-soundpad?style=flat-square)](https://github.com/arabianq/pipewire-soundpad/releases/latest)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20PipeWire-blue?style=flat-square)](https://pipewire.org/)

**PipeWire Soundpad (PWSP)** — это современное приложение с низкой задержкой, которое позволяет воспроизводить аудиофайлы прямо в ваш микрофон. Разработано специально для Linux с использованием мощи PipeWire для достижения нативной интеграции без использования мостов Pulseaudio или сложных виртуальных устройств.

---

## ✨ Возможности

- **Нативная интеграция с PipeWire:** Прямое взаимодействие с PipeWire API для минимальной задержки.
- **Модульная архитектура:** Состоит из фонового демона (`daemon`), интерфейса командной строки (`cli`) и графического интерфейса (`gui`).
- **Современный GUI:** Построен на `egui`, плавно работает как на Wayland, так и на X11.
- **Глобальные горячие клавиши:** Работают через `evdev`, позволяя воспроизводить звуки из любого окна.
- **Широкая поддержка форматов:** Использует `rodio` и `symphonia` для поддержки большинства аудиоформатов.

---

## 🚀 Установка

Мы предоставляем несколько способов установки PWSP, включая стабильные релизы и постоянно обновляемые "nightly" сборки напрямую из ветки `main`.

### 📦 Flatpak (Рекомендуется)

Добавьте наш официальный OSTree репозиторий и установите приложение:

```bash
# Добавьте репозиторий
flatpak remote-add --if-not-exists arabianq-repo https://arabianq.github.io/pipewire-soundpad/

# Установите стабильную (Stable) версию
flatpak install arabianq-repo ru.arabianq.pwsp//stable

# ИЛИ установите Nightly версию (самые свежие обновления)
flatpak install arabianq-repo ru.arabianq.pwsp//nightly
```

### 🟠 Debian / Ubuntu (APT Репозиторий)

Мы поддерживаем официальный APT-репозиторий для бесшовных обновлений через `apt`:

```bash
# 1. Скачайте публичный GPG ключ
wget -O- https://arabianq.github.io/pipewire-soundpad/apt/pubkey.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/pwsp.gpg

# 2. Добавьте репозиторий (Выберите STABLE или NIGHTLY)
# Для Stable:
echo "deb [signed-by=/etc/apt/keyrings/pwsp.gpg] https://arabianq.github.io/pipewire-soundpad/apt/ stable main" | sudo tee /etc/apt/sources.list.d/pwsp.list

# Для Nightly:
# echo "deb [signed-by=/etc/apt/keyrings/pwsp.gpg] https://arabianq.github.io/pipewire-soundpad/apt/ nightly main" | sudo tee /etc/apt/sources.list.d/pwsp.list

# 3. Обновите индексы и установите
sudo apt update
sudo apt install pwsp-gui
```

### 🐧 Fedora / RHEL (COPR)

Доступно через репозиторий Fedora COPR:

```bash
sudo dnf copr enable arabianq/pwsp
sudo dnf install pwsp
```

### ⚙️ Ручная установка

Вы можете вручную скачать пакеты `.deb` или готовые бинарники `.zip` на [странице релизов](https://github.com/arabianq/pipewire-soundpad/releases).

### 🦀 Сборка из исходников

Убедитесь, что у вас установлены Rust, Cargo и необходимые зависимости (`libpipewire-0.3-dev`, `libclang-dev`, `libasound2-dev`, `libdbus-1-dev`).

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

Вы также можете взаимодействовать с демоном напрямую через командную строку:

```bash
pwsp-cli play /path/to/sound.mp3
pwsp-cli stop
pwsp-cli status
```

---

## 📚 Документация и DeepWiki

Для детальной настройки, решения проблем, описания архитектуры и кастомных конфигураций, пожалуйста, посетите нашу официальную Wiki:

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
