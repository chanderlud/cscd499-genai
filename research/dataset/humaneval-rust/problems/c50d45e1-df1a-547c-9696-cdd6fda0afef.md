* **Spec:** /* Given two lists operator, and operand. The first list has basic algebra operations, and the second list is a list of integers. Use the two given lists to build the algebric expression and return the evaluation of this expression. The basic algebra operations: Addition ( + ) Subtraction ( - ) Multiplication ( * ) Floor division ( // ) Exponentiation ( ** ) Note: The length of operator list is equal to the length of operand list minus one. Operand is a list of of non-negative integers. Operator list has at least one operator, and operand list has at least two operands. *\
* **Constraints:**
* **Signature:**

```rust
use std::{slice::Iter, cmp::{max, self}, mem::replace, collections::{HashSet, HashMap}, ops::Index, ascii::AsciiExt};
use rand::Rng;
use regex::Regex;
use md5;
use std::any::{Any, TypeId};

fn do_algebra(operato: Vec<&str>, operand: Vec<i32>) -> i32 {
```
