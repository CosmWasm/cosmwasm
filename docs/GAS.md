# Gas

Gas is a way to measure computational expense of a smart contract execution,
including CPU time and storage cost. It's unit is 1, i.e. you can think of it as
countable points. Gas consumption is deterministic, so executing the same thing
costs the same amount of gas across all hardware and operating systems.

## CosmWasm gas vs. Cosmos SDK gas

CosmWasm charges gas for Wasm operations, calls to host functions and calls to
the Cosmos SDK. _CosmWasm gas_ is different from _Cosmos SDK gas_ as the numbers
here are much larger. Since we charge gas for arbitrary user defined operations,
we need to charge each Wasm operation individually and cannot group larger tasks
together. As a result, the gas values become much larger than in Cosmos SDK even
for very fast executions. There is a [multiplier][defaultgasmultiplier] to
translate between CosmWasm gas and Cosmos SDK. It was measured and set to 100 a
while ago and can be adjusted when necessary.

## CosmWasm gas pricing

For CosmWasm gas, the target gas consumption is 1 Teragas (10^12 gas) per
millisecond. This idea is [inspired by NEAR][neargas] and we encourage you to
read their excellent docs on that topic.

In order to meet this target, we execute Argon2 in a test contract ([#1120]).
This is a CPU and memory intense job that does not call out into the host. At a
constant gas cost per operation of 1 (pre CosmWasm 1.0), this consumed 96837752
gas and took 15ms on our CI system. The ideal cost per operation for this system
is `10**12 / (96837752 / 15)`: 154898. This is rounded to 150000 for simplicity.

Each machine is different, we know that. But the above target helps us in
multiple ways:

1. Develop an intuition what it means to burn X gas or how much gas can be used
   if a block should be executable in e.g. 1 second
2. Have a target for adjustments, e.g. when the Wasm runtime becomes faster or
   slower
3. Allow pricing of calls that are not executed in Wasm, such as crypto APIs
4. Find significant over or underpricing

[defaultgasmultiplier]:
  https://github.com/CosmWasm/wasmd/blob/v0.19.0/x/wasm/keeper/gas_register.go#L18
[neargas]: https://docs.near.org/docs/concepts/gas
[#1120]: https://github.com/CosmWasm/cosmwasm/pull/1120

## Gas overflow potential

CosmWasm gas aims for 1 Teragas/second, i.e. the uint64 range exceeds after 18
billion seconds (213 days)<sup>1</sup>. Assuming a max supported block execution
time of 100 seconds, the gas price has to be over-priced by a factor of 184
million (184 Petagas/second) in order to exceed the uint64 range<sup>2</sup>.
Since serious over or underpricing is considered a bug, using uint64 for gas
mesurments is considered safe.

Cosmos SDK gas uses values that are smaller by a factor of 150_000, so those
don't overflow as well. Since no Cosmos SDK gas values are processed inside of
this repository, this is not our main concern. However, it's good to know that
we can safely pass them in uint64 fields, as long as the full range is
supported. This is the case for the C API as well as
[JSON numbers](https://www.json.org/) as long as both sides support integers in
their JSON implementation. Go and Rust do that while many other implementations
don't support integers numbers and convert to them to IEEE-754 doubles, which
has a safe integer range up about 53 bit (e.g. JavaScript and jq).

<sup>1</sup> Python3: `(2**64-1) / 10**12`

<sup>2</sup> Python3: `((2**64-1)/100) / 10**12`
