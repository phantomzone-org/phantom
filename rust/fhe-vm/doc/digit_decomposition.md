The digit decomposition approach targets u32 values, but can be extended to u64 or u128 similarely. 

## Overview

The goal is to homomorphically split an u32 into a base B decomposition: given Enc(x) and B, we want to produce V = [Enc(x6), Enc(x5), Enc(x4), Enc(x3), Enc(x2), Enc(x1), Enc(x0)] such that $$\sum \textsf{Enc}(x)_{i}\cdot B^{i} $$

For example given B=32 and Enc(x) = 0xf0f0f0ff, then we want to output v = [Enc(3), Enc(24), Enc(15), Enc(1), Enc(28), Enc(7), Enc(31)], such that Enc(x) = 


11 11000 01111 00001 11100 00111 11111