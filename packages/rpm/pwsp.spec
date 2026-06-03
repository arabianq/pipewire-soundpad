# prevent library files from being installed
%global cargo_install_lib 0

# Fallback macros for systems without rpmautospec (e.g. openSUSE)
%{!?autorelease: %global autorelease 1}
%{!?autochangelog: %global autochangelog \
* Tue Jun 02 2026 Arabian <arabianq@github> - %{version}-%{release}\
- Release build}


# disable debuginfo package generation (debugsourcefiles.list is empty for Rust)
%global debug_package %{nil}


Name:            pwsp
Version:         1.11.0
Release:         %autorelease
Summary:         Lets you play audio files through your microphone

License:         MIT

URL:             https://github.com/arabianq/pipewire-soundpad
Source:          https://github.com/arabianq/pipewire-soundpad/archive/refs/tags/v%{version}.tar.gz

BuildRequires: rust
BuildRequires: cargo
BuildRequires: pipewire-devel
%if 0%{?suse_version}
BuildRequires: alsa-devel
BuildRequires: dbus-1-devel
%else
BuildRequires: alsa-lib-devel
BuildRequires: dbus-devel
%endif
BuildRequires: clang-devel
BuildRequires: cmake
BuildRequires: pkgconfig


%global _description %{expand:
PWSP lets you play audio files through your microphone. Has both CLI and
GUI clients.}

%description %{_description}

%prep
%autosetup -n pipewire-soundpad-%{version} -p1

%build
cargo build --release --locked

%install
install -Dm755 target/release/pwsp-cli %{buildroot}%{_bindir}/pwsp-cli
install -Dm755 target/release/pwsp-daemon %{buildroot}%{_bindir}/pwsp-daemon
install -Dm755 target/release/pwsp-gui %{buildroot}%{_bindir}/pwsp-gui

install -Dm644 pwsp-gui/assets/pwsp-gui.desktop %{buildroot}%{_datadir}/applications/pwsp.desktop
install -Dm644 pwsp-gui/assets/icon.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/pwsp.png

install -Dm644 pwsp-gui/assets/pwsp-daemon.service %{buildroot}/usr/lib/systemd/user/pwsp-daemon.service

%files
%license LICENSE
%doc README.md
%{_bindir}/pwsp-cli
%{_bindir}/pwsp-daemon
%{_bindir}/pwsp-gui
%{_datadir}/applications/pwsp.desktop
%{_datadir}/icons/hicolor/256x256/apps/pwsp.png
/usr/lib/systemd/user/pwsp-daemon.service

%changelog
%autochangelog
