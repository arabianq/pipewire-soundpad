#!/usr/bin/env python3

import argparse
import subprocess

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="PWSP Flatpak",
        add_help=True,
        exit_on_error=True
    )
    subparsers = parser.add_subparsers(dest="command")

    cli_parser = subparsers.add_parser("cli", add_help=False, prefix_chars=" ")
    cli_parser.add_argument("args", nargs=argparse.REMAINDER, help="Arguments for pwsp-cli")

    daemon_parser = subparsers.add_parser("daemon", add_help=True)
    daemon_group = daemon_parser.add_mutually_exclusive_group(required=True)
    daemon_group.add_argument("--start", action="store_true", help="Start pwps-daemon")
    daemon_group.add_argument("--kill", action="store_true", help="Kill pwsp-daemon")

    args = parser.parse_args()

    command = args.command
    if not command:
        subprocess.Popen("pwsp-daemon")
        subprocess.Popen("pwsp-gui")
    else:
        if command == "cli":
            subprocess.Popen(["pwsp-cli"] + args.args)
        elif command == "daemon":
            if args.start:
                subprocess.Popen("pwsp-daemon")
            elif args.kill:
                subprocess.Popen(["pkill", "f", "pwsp-daemon"])