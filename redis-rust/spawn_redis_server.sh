#!/bin/bash
set -o allexport && source .env && set +o allexport
cargo run
