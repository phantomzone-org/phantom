# End-to-end Examples of Phantom

This directory contains end-to-end examples of using Phantom.

* Template: Simple example to get started with Phantom. Evaluates an encrypted polynomial on the encrypted input.
* String Matching: Compares the encrypted input string with the string stored in memory.
* OTC Quote: OTC desk quote generation program on encrypted trade intent for the BTC/USD pair.
* Uniswap: Constant function automated market maker (CF-AMM) program that executes encrypted trade on encrypted pool. 

<!--
  cargo clean -p guest --target-dir /tmp/vm-experiments
-->

# Building from scratch

You can run `setup.sh` to build the project from scratch on a fresh Ubuntu/Debian machine.