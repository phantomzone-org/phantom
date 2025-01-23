## Overview

The goal is to homomorphically split an u32 into a base B decomposition: given $\textsf{Enc}(x)$ and $B$, we want to produce $\textsf{Enc}(x_{i})$ for $0\leq i<d$ where $d=\lceil32/\log_{2}(B)\rceil$ such that $$\sum \textsf{Enc}(x_{i})\cdot B^{i} = \textsf{Enc}(x)$$

For example given $B=32$ and $x = \texttt{0xf0f0f0ff}$, then we want $[\textsf{Enc}(3), \textsf{Enc}(24), \textsf{Enc}(15), \textsf{Enc}(1), \textsf{Enc}(28), \textsf{Enc}(7), \textsf{Enc}(31)]$.

## Algorithm

1) $i\leftarrow 0$
2) $x_{\texttt{2N}} \leftarrow x_{\textsf{u64}}\gg(64-(i+1)\cdot \log_{2}(B)-1) \mod 2N$: extracts the next batch of $\log_{2}(B)+1$ digits mod $Q$ and switches to mod $2N$.
3) $x_{\texttt{2N}} \leftarrow x_{\texttt{2N}} + 2^{\log_{2}(N) + \log_{2}(B) - 1}$: add $\frac{1}{2}\textsf{drift}$ to ensure that $x_{\texttt{2N}}\pm \frac{1}{2}\textsf{drift}$ is within the expected range.
4) $x_{\texttt{MSB}} \leftarrow \textsf{PBS}_{\texttt{MSB}}(x_{\texttt{2N}}): (x_{\texttt{2N}}\gg(\log_{2}(N)-1))\ll(\log_{2}(N)-1)$: PBS extracts MSB.
5) $x_{\texttt{2N}} \leftarrow x_{\texttt{2N}} - x_{\texttt{MSB}}$: subtracts MSB.
6) $x_{\texttt{u32}}^{(i)} \leftarrow \textsf{PBS}_{\texttt{DIGIT}}:(x_{\texttt{2N}}\gg(\log_{2}(N) - \log_{2}(B) - 1))\ll(\log_{2}(N) - \log_{2}(B)-1)$: PBS digit extraction.
7) $x_{\texttt{u32}}^{(i)} \leftarrow x_{\texttt{u32}}^{(i)} + x_{\texttt{MSB}}$ if $i=d-1$: adds back MSB if this is the last iteration.
8) $x_{\textsf{u64}} \leftarrow x_{\textsf{u64}} - x_{\texttt{u32}}^{(i)}\ll (63 - (i+1)\cdot\log_{2}(B) +1)$: removes the $\log_{2}(B)$ bottom digits.
9) $i\leftarrow i + 1$
10) If $i \neq d-1$, Go back to 3.

Note, if $B$ does not divides $32$, then the last iteration is slightly modified.