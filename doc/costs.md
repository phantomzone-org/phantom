```mermaid
graph TD

    ROM["<b>ROM read (size = 8KB):</b><br> - cost: 5ms"]

    DCP["<b>DCP[4xu5,1xu6]:</b>\n - decomposes 31 bits into: 4xu5, 1xu6\n - which requires 6 sequential DCP operations\n - each DCP operation requires 2 BRs\n - total 12 BRs required\n - Cost: 12 x 20ms = 240ms"]

    CBT1["<b>CBT[u5]:</b>\n - CBT with 5 bit value\n - multiple CBTs in parallel with depth of 1 CBT\n - Cost: 20ms"]

    ExternalLWERead["<b>Read 8xu4 LWEs with external product:</b>\n - Read multiple (in parallel) 8xu4 LWEs, rs1, rs2, etc. from register poly, imm poly\n - plaintext is stored as 8 limbs each consisting 4 bits\n - hence, reading single 8xu8 requires 8 external products (in parallel)\n - Cost: 1 external product = 5ms"]

    ArithmeticRoutine["<b>Arithmetic routine:</b>\n - Perform all arithmetic operations in parallel\n - cost equals of the most expensive operation\n - Add/Sub: 100ms\n - Mul: 200ms\n -Div/Rem: 3.5s\n - Cost (rvi32): 100ms\n Cost (rvi32m): 3.5s"]

    UpdateRD["<b>Update RD:</b>\n - update register RD\n - recall: registers fit into single polynomial \n - Cost: 10ms"]

    RamReadWrite["<b>RAM (size = 8KB) read, write:</b>\n - Computer address ADD, read ADD, then write to ADD\n - requires 1xu13 addition to compute ADD (32ms) - requires 4 serial DCP to extract 3x3 bits, 1x2 bits\n - each DCP requires 2 BRs\n - requires 4 CBTs in parallel, hence depth of 1 CBT\n - each CBT requires 1BR\n - RAM fits in single polynomial, \hence read = 1 external product (5ms) and write = 1 external product (5ms) + 11 K.S. (5ms)\n  - Cost: 32ms + 8x20 ms + 20ms + 5ms + 10ms = 227ms"]

    BrancOpsPCUpdate["<b>Execute branching operations and PC update:</b>\n - Exectue all branching operations (in parallel) and update PC\n - Then CBT PC to select next instruction from ROM\n - Branching operation requires u32 conditional following by selection, thus takes 80ms\n - CBT PC requires 4 serial DCPs and 4 (in parallel) CBTs - Cost: 80ms + 8x20ms + 20ms = 260ms"]


    ROM --> DCP
    DCP --> CBT1
    CBT1 --> ExternalLWERead
    ExternalLWERead --> ArithmeticRoutine
    ArithmeticRoutine --> UpdateRD
    ExternalLWERead --> RamReadWrite
    ExternalLWERead --> BrancOpsPCUpdate

    style DCP text-align:left
    style CBT1 text-align:left
    style ExternalLWERead text-align:left
    style ArithmeticRoutine text-align:left
    style UpdateRD text-align:left
    style BrancOpsPCUpdate text-align:left
    style RamReadWrite text-align:left
    style ROM text-align:left
```

Dependency graph of operations in risc-v FHE-VM. A single cycle starts at top and ends at the bottom. Operation blocks at same level are processed in parallel, thus the total cycle time equals summation of time take by each block on longest path.

<br>

-   Note that the figures are rough estimations and runtime in practice will deviate.
-   The time per cycle is total time of the longest path.
    -   Single cycle cost for rv32i = 530ms
    -   Single cycle cost for rv32im (i.e. with "M" extension) = 4010ms
        -   u32 Div/Rem instructions take approx. 3.5s.
-   Reference notes:
    -   BR stands for blind rotation. Blind rotation approx. takes 20ms on CPU.
    -   CBT stands for circuit bootstrapping. CBT requires $d$ blind rotations, where $d$ is decomposition count of desired RGSW ciphertext. BRs of single CBT can be processed in parallel
    -   Cost for arithmetic operations on u32 are taken from this [link](https://docs.zama.ai/tfhe-rs/get-started/benchmarks).
-   Questions:
    -   Are there alternative integer representations in which Div/Rem are less expensive?
