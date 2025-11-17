<!-- 
- Use PC to read instruction components from ROM (IMM, RS1_INDEX, RS2_INDEX, RD_INDEX, PCU, RDU, MU)
- Read registers (RS1_INDEX -> RS1, RS2_INDEX -> RS2)
- Prepare all values for evaluation (IMM, RS1, RS2)
- Compute ram address (IMM, RS1 -> RAM_ADDRESS) and read ram at ram address (RAM_ADDRESS, RAM -> RAM_VAL)
- Compute possible new value of RD (IMM, RS1, RS2, IMM, PC, RAM_VAL -> POSSIBLE_RD_VALS)
- Blind select correct rd value (RDU, POSSIBLE_RD_VALS -> NEW_RD_VAL)
- Put the rd value in rd (RD_INDEX, NEW_RD_VAL -> RD)
-  -->



<!--
Average Cycle Time: 1.55773912s
  1. Read instruction components: 128.317183ms
  2. Read registers: 141.108799ms
  3. Prepare imm rs1 rs2 values: 321.63368ms
  4. Read ram: 98.038346ms
  5. Update registers: 549.759557ms
     - Evaluate rd ops: 306.301296ms
     - Blind selection: 66.950837ms
     - Write rd: 176.506174ms
  6. Update ram: 197.162751ms
  7. Update pc: 121.582101ms
     - PCU prepare: 62.835836ms
     - PC update BDD: 58.745625ms
-->

Below is the dependency of one cycle in Phantom.
It shows how each intermediate value in Phantom is computed and which other values it depends on.
A single cycle starts at top and ends at the bottom.
Operation blocks at same level are processed in parallel, thus the total cycle time equals summation of time take by each block on longest path.

The runtimes are from running Phantom on a AWS m6a.8xlarge, parallelized with 32 threads.
Runtimes are subject to improvement and may vary, depending on the hardware.


```mermaid
graph TD

    subgraph start_block [" "]
        PC["PC "]
        ROM["ROM"]
        RAM["RAM"]
        REGISTERS["REGISTERS"]
    end

    subgraph read_inst [" "]
        READ_INST_LABEL["Read<br>Instruction<br>Components<br>(128 ms)"]
        IMM["IMM"]
        RS1_INDEX["RS1_INDEX"]
        RS2_INDEX["RS2_INDEX"]
        RD_INDEX["RD_INDEX"]
        PCU["PCU"]
        RDU["RDU"]
        MU["MU"]
    end
    
    subgraph read_ram [" "]
        READ_RAM_LABEL["Read RAM<br>(98 ms)"]
        RAM_ADDRESS["RAM_ADDRESS"]
        RAM_VAL["RAM_VAL"]
    end

    subgraph read_reg [" "]
        READ_REG_LABEL["Read Registers<br>(141 ms)"]
        RS1["RS1"]
        RS2["RS2"]
    end

    subgraph update_reg [" "]
        UPDATE_REG_LABEL["Update Registers<br>(549 ms)"]
        POSSIBLE_RD_VALS["POSSIBLE_RD_VALS"]
        RD["RD"]
    end


    subgraph update_ram [" "]
        UPDATE_RAM_LABEL["Update RAM<br>(197 ms)"]
        POSSIBLE_RAM_VALS["POSSIBLE_RAM_VALS"]
        NEW_RAM_VAL["NEW_RAM_VAL"]
    end

    subgraph update_pc [" "]
        UPDATE_PC_LABEL["Update PC<br>(121 ms)"]
        NEW_PC_VAL["NEW_PC_VAL"]
    end

    subgraph end_block [" "]
        PC_AFTER["PC"]
        RAM_AFTER["RAM"]
        REGISTERS_AFTER["REGISTERS"]
    end


    PC --> IMM
    PC --> RS1_INDEX
    PC --> RS2_INDEX
    PC --> RD_INDEX
    PC --> PCU
    PC --> RDU
    PC --> MU

    ROM --> IMM
    ROM --> RS1_INDEX
    ROM --> RS2_INDEX
    ROM --> RD_INDEX
    ROM --> PCU
    ROM --> RDU
    ROM --> MU    

    RS1_INDEX --> RS1
    RS2_INDEX --> RS2

    REGISTERS --> RS1
    REGISTERS --> RS2
    IMM --> RAM_ADDRESS
    RS1 --> RAM_ADDRESS

    RAM_ADDRESS --> RAM_VAL
    RAM --> RAM_VAL

    IMM --> POSSIBLE_RD_VALS
    RS1 --> POSSIBLE_RD_VALS
    RS2 --> POSSIBLE_RD_VALS
    PC --> POSSIBLE_RD_VALS
    RAM_VAL --> POSSIBLE_RD_VALS

    POSSIBLE_RD_VALS --> RD
    RD_INDEX --> RD
    RDU --> RD

    RS2 --> POSSIBLE_RAM_VALS
    RAM_VAL --> POSSIBLE_RAM_VALS
    RAM_ADDRESS --> POSSIBLE_RAM_VALS
    RS2 --> POSSIBLE_RAM_VALS
    
    POSSIBLE_RAM_VALS --> NEW_RAM_VAL
    MU --> NEW_RAM_VAL


    IMM --> NEW_PC_VAL
    RS1 --> NEW_PC_VAL
    RS2 --> NEW_PC_VAL
    PC --> NEW_PC_VAL
    PCU --> NEW_PC_VAL

    NEW_PC_VAL --> PC_AFTER
    RD --> REGISTERS_AFTER
    NEW_RAM_VAL --> RAM_AFTER

    classDef labelStyle stroke-dasharray: 5 5, font-weight: bold, font-size: 120%
    class READ_INST_LABEL labelStyle
    class READ_RAM_LABEL labelStyle
    class READ_REG_LABEL labelStyle
    class UPDATE_REG_LABEL labelStyle
    class UPDATE_RAM_LABEL labelStyle
    class UPDATE_PC_LABEL labelStyle
```


## Summary of runtime
The runtimes are from running Phantom on a AWS m6a.8xlarge, parallelized with 32 threads.

Average Cycle Time: 1.55773912s
  1. Read instruction components: 128.317183ms
  2. Read registers: 141.108799ms
  3. Prepare imm rs1 rs2 values: 321.63368ms
  4. Read ram: 98.038346ms
  5. Update registers: 549.759557ms
     <!-- - Evaluate rd ops: 306.301296ms
     - Blind selection: 66.950837ms
     - Write rd: 176.506174ms -->
  6. Update ram: 197.162751ms
  7. Update pc: 121.582101ms
     <!-- - PCU prepare: 62.835836ms
     - PC update BDD: 58.745625ms -->


<!-- 
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
    -   Are there alternative integer representations in which Div/Rem are less expensive? -->
