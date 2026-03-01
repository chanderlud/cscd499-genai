* **Spec:** /* Given a positive integer n, you have to make a pile of n levels of stones. The first level has n stones. The number of stones in the next level is: - the next odd number if n is odd. - the next even number if n is even. Return the number of stones in each level in a list, where element at index i represents the number of stones in the level (i+1). *\
* **Constraints:**
* **Signature:**

```rust
use std::{slice::Iter, cmp::{max, self}, mem::replace, collections::{HashSet, HashMap}, ops::Index, ascii::AsciiExt};
use rand::Rng;
use regex::Regex;
use md5;
use std::any::{Any, TypeId};

fn make_a_pile(n:i32) -> Vec<i32>{
```
