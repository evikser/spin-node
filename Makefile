r:
	cd example_contracts && sudo cargo +nightly-2023-03-06 build --release
	cp example_contracts/target/riscv-guest/riscv32im-risc0-zkvm-elf/release/token_contract spin_core/state/contracts/token.spin
	cp example_contracts/target/riscv-guest/riscv32im-risc0-zkvm-elf/release/fibonacci_contract spin_core/state/contracts/fibonacci.spin
	cd spin_core && sudo cargo +nightly-2023-03-06 run --release --bin example_token

v:
	cd verifier && npx hardhat test

c:
	cd example_contracts && sudo cargo clean