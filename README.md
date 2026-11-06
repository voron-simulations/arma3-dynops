# DynamicOperations

[![Nightly](https://github.com/voron-simulations/arma3-dynops/actions/workflows/nightly.yml/badge.svg)](https://github.com/voron-simulations/arma3-dynops/actions/workflows/nightly.yml)

Mission generation framework for ArmA 3. Aims to provide a set of scripts/modules which create convincing mission background in form of AI population

## Components

- **Location detection** - settlement detection from building positions, exposed to SQF as data and as `Location` objects. See [`docs/locations.md`](docs/locations.md).
- **Population** - budget-driven allocation of AI groups over faction-owned locations, computed in the extension and spawned from SQF. See [`docs/population.md`](docs/population.md).

Design decisions are recorded in [`adr/`](adr/).
