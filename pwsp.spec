%bcond check 1

# prevent library files from being installed
%global cargo_install_lib 0

Name:            pwsp
Version:         1.0.1
Release:         %autorelease
Summary:         Lets you play audio files through your microphone

License:         MIT

URL:             https://github.com/arabianq/pipewire-soundpad
Source:          https://github.com/arabianq/pipewire-soundpad/archive/refs/tags/v%{version}.tar.gz

BuildRequires: rust
BuildRequires: cargo
BuildRequires: pipewire-devel
BuildRequires: alsa-lib-devel
BuildRequires: clang-devel

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

install -Dm644 assets/pwsp-gui.desktop %{buildroot}%{_datadir}/applications/pwsp.desktop
install -Dm644 assets/icon.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/pwsp.png

install -Dm644 assets/pwsp-daemon.service %{buildroot}/usr/lib/systemd/user/pwsp-daemon.service

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