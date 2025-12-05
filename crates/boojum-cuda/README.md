# zkSync Era: A ZK Rollup For Scaling Ethereum

ADI Stack is a layer 2 rollup that uses zero-knowledge proofs to scale Ethereum without compromising on security or
decentralization.

## Boojum-CUDA

Boojum-CUDA is a library implementing GPU-accelerated cryptographic functionality for the zkSync prover.

Prerequisites:

- CUDA Toolkit 12.x
- CMake 3.24 and up
- clang
- rust nightly toolchain

By default, the CUDA code is compiled for the GPU that is present in the system. If there is no GPU in the system or
another architecture is desired, the environment variable `CUDAARCHS` can be set to the desired architecture.
See https://cmake.org/cmake/help/latest/variable/CMAKE_CUDA_ARCHITECTURES.html.

## Policies

- [Contribution policy](CONTRIBUTING.md)

## License

zkSync Era is distributed under the terms of either

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Official Links

- [Website](https://adi.foundation)
- [Docs](https://docs.adi.foundation/)
- [Github](https://github.com/ADI-Foundation-Labs/)
- [X](https://x.com/adi_foundation)
- [X for ADI Chain announcements](https://x.com/ADIChain_)
- [LinkedIn](https://www.linkedin.com/company/adifoundation/)
- [Discord](http://discord.gg/adi-foundation)

## Disclaimer

zkSync Era has been through lots of testing and audits. Although it is live, it is still in alpha state and will go
through more audits and bug bounties programs. We would love to hear our community's thoughts and suggestions about it!
It is important to state that forking it now can potentially lead to missing important security updates, critical
features, and performance improvements.
