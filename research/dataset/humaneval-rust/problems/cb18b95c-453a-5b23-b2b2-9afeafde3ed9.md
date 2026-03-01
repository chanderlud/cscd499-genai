* **Spec:** /* Given a string, find out how many distinct characters (regardless of case) does it consist of *\
* **Constraints:**
* **Signature:**

```rust
use std::{slice::Iter, cmp::{max, self}, mem::replace, collections::{HashSet, HashMap}, ops::Index, ascii::AsciiExt};
use rand::Rng;
use regex::Regex;
use md5;
use std::any::{Any, TypeId};

fn count_distinct_characters(str:String) -> i32{
```
