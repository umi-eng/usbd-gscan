# `usbd-gscan`

[![docs.rs](https://docs.rs/usbd-gscan/badge.svg)](https://docs.rs/usbd-gscan)
[![crates.io](https://img.shields.io/crates/v/usbd-gscan.svg)](https://crates.io/crates/usbd-gscan)
[![codecov](https://codecov.io/github/umi-eng/usbd-gscan/graph/badge.svg?token=7VLM5K07RC)](https://codecov.io/github/umi-eng/usbd-gscan)

An implementation of the Geschwister Schneider USB/CAN protocol.

## Features

- Classic CAN and CAN FD frames.
- FD bitrate switching configuration.
- Error reporting frames.
- Outgoing frame buffer.
- Hardware timestamp.
- Windows support with WinUSB descriptors.

## Limitations

- Only supports a maximum of 3 interfaces as per the Linux kernel implementation.
