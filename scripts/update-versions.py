#!/usr/bin/env python3
import sys
import os
import re
import subprocess
import shutil
from datetime import datetime


# Helper to print errors and exit
def fatal(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


# Get the root directory of the project
root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root_dir)

# Read current version from Cargo.toml
cargo_toml_path = "Cargo.toml"
if not os.path.exists(cargo_toml_path):
    fatal("Cargo.toml not found in the root directory.")

with open(cargo_toml_path, "r", encoding="utf-8") as f:
    cargo_toml_content = f.read()

# We want to match version in [workspace.package]
# First, let's find the [workspace.package] section
workspace_package_match = re.search(
    r"\[workspace\.package\](.*?)(?=\n\[|$)", cargo_toml_content, re.DOTALL
)
if not workspace_package_match:
    fatal("Could not find [workspace.package] section in Cargo.toml.")

workspace_package_sec = workspace_package_match.group(1)
version_match = re.search(r'version\s*=\s*"([^"]+)"', workspace_package_sec)
if not version_match:
    fatal("Could not find version in [workspace.package] in Cargo.toml.")

current_version = version_match.group(1)
print(f"Current version detected: {current_version}")

# Get new version
if len(sys.argv) < 2:
    try:
        new_version = input(f"Enter new version: ").strip()
    except (KeyboardInterrupt, EOFError):
        print()
        sys.exit(0)
    if not new_version:
        fatal("No version provided.")
else:
    new_version = sys.argv[1].strip()

if not re.match(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$", new_version):
    fatal(f"Invalid version format: '{new_version}'. Should be like '1.10.1'.")

# 1. Update Cargo.toml
print("Updating Cargo.toml...")


def replace_version_in_workspace(match):
    section_content = match.group(1)
    updated_section_content = re.sub(
        r'(version\s*=\s*")[^"]+(")', rf"\g<1>{new_version}\g<2>", section_content
    )
    return f"[workspace.package]{updated_section_content}"


new_cargo_toml = re.sub(
    r"\[workspace\.package\](.*?)(?=\n\[|$)",
    replace_version_in_workspace,
    cargo_toml_content,
    flags=re.DOTALL,
)

with open(cargo_toml_path, "w", encoding="utf-8") as f:
    f.write(new_cargo_toml)

# Update Cargo.lock using cargo
print("Updating Cargo.lock using cargo generate-lockfile...")
try:
    subprocess.run(["cargo", "generate-lockfile"], check=True)
except Exception as e:
    print(f"Warning: Failed to update Cargo.lock using cargo: {e}")

# 2. Update packages/aur/bin/PKGBUILD
pkgbuild_bin_path = "packages/aur/bin/PKGBUILD"
if os.path.exists(pkgbuild_bin_path):
    print(f"Updating {pkgbuild_bin_path}...")
    with open(pkgbuild_bin_path, "r", encoding="utf-8") as f:
        content = f.read()
    content = re.sub(r"pkgver=[^\n]+", f"pkgver={new_version}", content)
    content = re.sub(r"pkgrel=[^\n]+", "pkgrel=1", content)
    with open(pkgbuild_bin_path, "w", encoding="utf-8") as f:
        f.write(content)

# 3. Update packages/aur/standart/PKGBUILD
pkgbuild_std_path = "packages/aur/standart/PKGBUILD"
if os.path.exists(pkgbuild_std_path):
    print(f"Updating {pkgbuild_std_path}...")
    with open(pkgbuild_std_path, "r", encoding="utf-8") as f:
        content = f.read()
    content = re.sub(r"pkgver=[^\n]+", f"pkgver={new_version}", content)
    content = re.sub(r"pkgrel=[^\n]+", "pkgrel=1", content)
    with open(pkgbuild_std_path, "w", encoding="utf-8") as f:
        f.write(content)


# Update AUR .SRCINFO files
def update_srcinfo(directory, pkgbuild_path, srcinfo_path):
    if not os.path.exists(srcinfo_path):
        return
    print(f"Updating {srcinfo_path}...")
    if shutil.which("makepkg"):
        try:
            print(f"Running makepkg --printsrcinfo in {directory}...")
            result = subprocess.run(
                ["makepkg", "--printsrcinfo"],
                cwd=directory,
                capture_output=True,
                text=True,
                check=True,
            )
            with open(srcinfo_path, "w", encoding="utf-8") as f:
                f.write(result.stdout)
            return
        except Exception as e:
            print(
                f"Warning: makepkg failed in {directory}: {e}. Falling back to text replacement."
            )

    # Text replacement fallback
    with open(srcinfo_path, "r", encoding="utf-8") as f:
        content = f.read()
    content = re.sub(r"pkgver\s*=\s*[^\n]+", f"pkgver = {new_version}", content)
    content = re.sub(r"pkgrel\s*=\s*[^\n]+", "pkgrel = 1", content)
    content = content.replace(current_version, new_version)
    with open(srcinfo_path, "w", encoding="utf-8") as f:
        f.write(content)


update_srcinfo("packages/aur/bin", pkgbuild_bin_path, "packages/aur/bin/.SRCINFO")
update_srcinfo(
    "packages/aur/standart", pkgbuild_std_path, "packages/aur/standart/.SRCINFO"
)

# 4. Update packages/flatpak/ru.arabianq.pwsp.metainfo.xml
flatpak_xml_path = "packages/flatpak/ru.arabianq.pwsp.metainfo.xml"
if os.path.exists(flatpak_xml_path):
    print(f"Updating {flatpak_xml_path}...")
    with open(flatpak_xml_path, "r", encoding="utf-8") as f:
        content = f.read()

    today_str = datetime.today().strftime("%Y-%m-%d")
    content = re.sub(
        r'<release\s+version="[^"]+"\s+date="[^"]+"\s*/?>',
        f'<release version="{new_version}" date="{today_str}" />',
        content,
    )
    with open(flatpak_xml_path, "w", encoding="utf-8") as f:
        f.write(content)

# 5. Update packages/rpm/pwsp.spec
rpm_spec_path = "packages/rpm/pwsp.spec"
if os.path.exists(rpm_spec_path):
    print(f"Updating {rpm_spec_path}...")
    with open(rpm_spec_path, "r", encoding="utf-8") as f:
        content = f.read()
    content = re.sub(r"Version:\s*[^\n]+", f"Version:         {new_version}", content)
    with open(rpm_spec_path, "w", encoding="utf-8") as f:
        f.write(content)

print(f"Successfully updated all versions to {new_version}!")
