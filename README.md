# Monoio Route

Yet another high-performance route library written in Rust.

```rust
use monoio_route::Tree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Tree::new();
    router.insert(b"/home", "Welcome!")?;
    router.insert(b"/users/:id", "A User")?;
    router.insert(b"/message/:id", "A Message")?;
    router.insert(b"/:module/:id", "Other Module")?;
    router.insert(b"/*any", "Other Path")?;

    let (val, params) = router.at(b"/users/978").unwrap();
    assert_eq!(params.iter().find(|(k, _)| k == b"id").unwrap().1, b"978");
    assert_eq!(*val, "A User");

    let (val, params) = router.at(b"/some_module/978").unwrap();
    assert_eq!(params.iter().find(|(k, _)| k == b"id").unwrap().1, b"978");
    assert_eq!(params.iter().find(|(k, _)| k == b"module").unwrap().1, b"some_module");
    assert_eq!(*val, "Other Module");

    let (val, _) = router.at(b"/typo").unwrap();
    assert_eq!(*val, "Other Path");

    Ok(())
}
```

Like the well-known matchit, but different in:
- Support register param route and catch all route at the same path.
    - Matching priority: static > param > regex > catch_all
- Support regex route(in the future).
- Not support bare param or catch all.
- Syntax changed to style like go httproute(`{param}`->`:param`, `{*any}`->`*any`).
