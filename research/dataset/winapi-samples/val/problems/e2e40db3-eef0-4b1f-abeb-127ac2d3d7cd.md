N) Parse XML with XmlDocument

**Spec:** Write a function that loads an XML string using `XmlDocument` and returns the root element's node name and inner text as strings.

**Constraints:**
- Use the `windows` crate and the `XmlDocument` from the Windows.Data.Xml.Dom namespace.
- The function must return a `windows::core::Result` containing a tuple of the root element's node name and inner text as `String`s.

**Signature:**
```rust
fn parse_xml(xml: &str) -> windows::core::Result<(String, String)>
```

**Example:**
```rust
let (name, text) = parse_xml("<html>hello world</html>")?;
assert_eq!(name, "html");
assert_eq!(text, "hello world");
```