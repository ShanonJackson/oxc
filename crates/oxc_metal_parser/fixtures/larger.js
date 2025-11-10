// Larger JS fixture (subset-friendly). Focus on lets, numbers, strings, arithmetic.
// Avoids functions and loops to match early parser slice for correctness gating.

"use strict";

let total = 0; let i = 0; i = i + 1; total = total + i;

// Repeated simple blocks
let acc0 = 0; let s0 = "alpha0"; acc0 = acc0 + (2 * 3) - 5; // c0
let acc1 = 1; let s1 = "alpha1"; acc1 = acc1 + (3 * 4) - 6; // c1
let acc2 = 2; let s2 = "alpha2"; acc2 = acc2 + (4 * 5) - 7; // c2
let acc3 = 3; let s3 = "alpha3"; acc3 = acc3 + (5 * 6) - 8; // c3
let acc4 = 4; let s4 = "alpha4"; acc4 = acc4 + (6 * 7) - 9; // c4
let acc5 = 5; let s5 = "alpha5"; acc5 = acc5 + (7 * 8) - 10; // c5
let acc6 = 6; let s6 = "alpha6"; acc6 = acc6 + (8 * 9) - 11; // c6
let acc7 = 7; let s7 = "alpha7"; acc7 = acc7 + (9 * 10) - 12; // c7
let acc8 = 8; let s8 = "alpha8"; acc8 = acc8 + (10 * 11) - 13; // c8
let acc9 = 9; let s9 = "alpha9"; acc9 = acc9 + (11 * 12) - 14; // c9

let sum = 0; let j = 0;
sum = sum + acc0 + acc1 + acc2 + acc3 + acc4 + acc5 + acc6 + acc7 + acc8 + acc9;
sum = sum + (j * 3) + (j + 4) - (j - 5);
let str = "line" + j + ": value=" + sum;
// Comment with numbers 12345 and identifiers abcDEF_$

// A few parens and arithmetic (with comma expressions in parens)
let z = (1 + 2) * (3 + 4, 7) / (5 - 6) + (7 * 8) - (9 / 10);

// End marker string
"end-of-larger-fixture";

