#!/usr/bin/env bash
set -e

export GOOS=linux
export GOARCH=mipsle
export GOMIPS=softfloat
go build -tags example ./cmd/keeper


