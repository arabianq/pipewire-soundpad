#!/bin/sh

pwsp-daemon &
exec pwsp-gui "$@"
