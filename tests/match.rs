//! The test cases are mainly converted from matchit's test.

macro_rules! assert_at {
    ($matched:expr, $expected_val:expr, $expected_params:expr) => {{
        let (val, params) = $matched;
        assert_eq!(val, &&$expected_val);
        assert_eq!(params.len(), $expected_params.len());
        for (i, (name, value)) in $expected_params.iter().enumerate() {
            assert_eq!(&params[i].0, name);
            assert_eq!(&params[i].1, value);
        }
    }};
}

macro_rules! slices {
    ($($name:expr),*) => {
        &[$($name.as_bytes()),*]
    };
    ($($name:expr,)*) => {
        &[$($name.as_bytes()),*]
    };
}

macro_rules! slice_pairs {
    ($($name:expr => $value:expr),*) => {
        &[$(($name.as_bytes(), $value.as_bytes())),*].as_slice()
    };
}

macro_rules! test_tree {
    ($routes:expr, $matches:expr) => {{
        let mut tree = monoio_route::Tree::new();
        let routes: &[&[u8]] = $routes;
        for path in routes {
            assert!(tree.insert(path, path).is_ok());
        }
        for (path, val, params) in $matches {
            let val: Option<&str> = val;
            let params: &[(&[u8], &[u8])] = params;
            match tree.at(path.as_bytes()) {
                Some(matched) => {
                    let params: &[(&[u8], &[u8])] = params;
                    assert_at!(matched, val.unwrap().as_bytes(), params);
                }
                None => {
                    assert!(val.is_none())
                }
            }
        }
    }};
}

#[test]
fn partial_overlap() {
    test_tree!(
        slices!["/foo_bar", "/foo/bar", "/foo"],
        [("/foo/", None, slice_pairs!())]
    );
}

#[test]
fn wildcard_overlap() {
    test_tree!(
        slices!["/path/foo", "/path/*rest"],
        [
            ("/path/foo", Some("/path/foo"), slice_pairs!()),
            (
                "/path/bar",
                Some("/path/*rest"),
                slice_pairs!("rest" => "bar")
            ),
            (
                "/path/foo/",
                Some("/path/*rest"),
                slice_pairs!("rest" => "foo/")
            ),
        ]
    );

    test_tree!(
        slices!["/path/foo/:arg", "/path/*rest"],
        [
            (
                "/path/foo/myarg",
                Some("/path/foo/:arg"),
                slice_pairs!("arg"=>"myarg")
            ),
            (
                "/path/foo/myarg/",
                Some("/path/*rest"),
                slice_pairs!("rest" => "foo/myarg/")
            ),
            (
                "/path/foo/myarg/bar/baz",
                Some("/path/*rest"),
                slice_pairs!("rest" => "foo/myarg/bar/baz")
            ),
        ]
    );
}

#[test]
fn overlapping_param_backtracking() {
    test_tree!(
        slices!["/:object/:id", "/secret/:id/path"],
        [
            (
                "/secret/978/path",
                Some("/secret/:id/path"),
                slice_pairs!("id"=>"978")
            ),
            (
                "/something/978",
                Some("/:object/:id"),
                slice_pairs!("object" => "something", "id" => "978")
            ),
            (
                "/secret/978",
                Some("/:object/:id"),
                slice_pairs!("object" => "secret", "id" => "978")
            ),
        ]
    );
}

// This test will not pass since we do not support it.
// #[test]
// fn bare_catchall() {
//     test_tree!(
//         slices!["*foo", "foo/*bar"],
//         [
//             ("x/y", Some("*foo"), slice_pairs! { "foo" => "x/y" }),
//             ("/x/y", Some("*foo"), slice_pairs! { "foo" => "/x/y" }),
//             (
//                 "/foo/x/y",
//                 Some("*foo"),
//                 slice_pairs! { "foo" => "/foo/x/y" }
//             ),
//             ("foo/x/y", Some("foo/*bar"), slice_pairs! { "bar" => "x/y" }),
//         ]
//     );
// }

#[test]
fn normalized() {
    test_tree!(
        slices![
            "/x/:foo/bar",
            "/x/:bar/baz",
            "/:foo/:baz/bax",
            "/:foo/:bar/baz",
            "/:fod/:baz/:bax/foo",
            "/:fod/baz/bax/foo",
            "/:foo/baz/bax",
            "/:bar/:bay/bay",
            "/s",
            "/s/s",
            "/s/s/s",
            "/s/s/s/s",
            "/s/s/:s/x",
            "/s/s/:y/d"
        ],
        [
            (
                "/x/foo/bar",
                Some("/x/:foo/bar"),
                slice_pairs! { "foo" => "foo" }
            ),
            (
                "/x/foo/baz",
                Some("/x/:bar/baz"),
                slice_pairs! { "bar" => "foo" }
            ),
            (
                "/y/foo/baz",
                Some("/:foo/:bar/baz"),
                slice_pairs! { "foo" => "y", "bar" => "foo" },
            ),
            (
                "/y/foo/bax",
                Some("/:foo/:baz/bax"),
                slice_pairs! { "foo" => "y", "baz" => "foo" },
            ),
            (
                "/y/baz/baz",
                Some("/:foo/:bar/baz"),
                slice_pairs! { "foo" => "y", "bar" => "baz" },
            ),
            (
                "/y/baz/bax/foo",
                Some("/:fod/baz/bax/foo"),
                slice_pairs! { "fod" => "y" }
            ),
            (
                "/y/baz/b/foo",
                Some("/:fod/:baz/:bax/foo"),
                slice_pairs! { "fod" => "y", "baz" => "baz", "bax" => "b" },
            ),
            (
                "/y/baz/bax",
                Some("/:foo/baz/bax"),
                slice_pairs! { "foo" => "y" }
            ),
            (
                "/z/bar/bay",
                Some("/:bar/:bay/bay"),
                slice_pairs! { "bar" => "z", "bay" => "bar" },
            ),
            ("/s", Some("/s"), slice_pairs! {}),
            ("/s/s", Some("/s/s"), slice_pairs! {}),
            ("/s/s/s", Some("/s/s/s"), slice_pairs! {}),
            ("/s/s/s/s", Some("/s/s/s/s"), slice_pairs! {}),
            ("/s/s/s/x", Some("/s/s/:s/x"), slice_pairs! { "s" => "s" }),
            ("/s/s/s/d", Some("/s/s/:y/d"), slice_pairs! { "y" => "s" }),
        ]
    );
}

#[test]
fn blog() {
    test_tree!(
        slices![
            "/:page",
            "/posts/:year/:month/:post",
            "/posts/:year/:month/index",
            "/posts/:year/top",
            "/static/*path",
            "/favicon.ico"
        ],
        [
            ("/about", Some("/:page"), slice_pairs! { "page" => "about" }),
            (
                "/posts/2021/01/rust",
                Some("/posts/:year/:month/:post"),
                slice_pairs! { "year" => "2021", "month" => "01", "post" => "rust" },
            ),
            (
                "/posts/2021/01/index",
                Some("/posts/:year/:month/index"),
                slice_pairs! { "year" => "2021", "month" => "01" },
            ),
            (
                "/posts/2021/top",
                Some("/posts/:year/top"),
                slice_pairs! { "year" => "2021" },
            ),
            (
                "/static/foo.png",
                Some("/static/*path"),
                slice_pairs! { "path" => "foo.png" },
            ),
            ("/favicon.ico", Some("/favicon.ico"), slice_pairs! {}),
        ]
    );
}

#[test]
fn double_overlap() {
    test_tree!(
        slices![
            "/:object/:id",
            "/secret/:id/path",
            "/secret/978",
            "/other/:object/:id/",
            "/other/an_object/:id",
            "/other/static/path",
            "/other/long/static/path/"
        ],
        [
            (
                "/secret/978/path",
                Some("/secret/:id/path"),
                slice_pairs! { "id" => "978" }
            ),
            (
                "/some_object/978",
                Some("/:object/:id"),
                slice_pairs! { "object" => "some_object", "id" => "978" }
            ),
            ("/secret/978", Some("/secret/978"), slice_pairs! {}),
            ("/super_secret/978/", None, slice_pairs! {}),
            (
                "/other/object/1/",
                Some("/other/:object/:id/"),
                slice_pairs! { "object" => "object", "id" => "1" }
            ),
            ("/other/object/1/2", None, slice_pairs! {}),
            (
                "/other/an_object/1",
                Some("/other/an_object/:id"),
                slice_pairs! { "id" => "1" }
            ),
            (
                "/other/static/path",
                Some("/other/static/path"),
                slice_pairs! {}
            ),
            (
                "/other/long/static/path/",
                Some("/other/long/static/path/"),
                slice_pairs! {},
            ),
        ]
    );
}

#[test]
fn catchall_off_by_one() {
    test_tree!(
        slices!["/foo/*catchall", "/bar", "/bar/", "/bar/*catchall"],
        [
            ("/foo", None, slice_pairs! {}),
            ("/foo/", None, slice_pairs! {}),
            (
                "/foo/x",
                Some("/foo/*catchall"),
                slice_pairs! { "catchall" => "x" }
            ),
            ("/bar", Some("/bar"), slice_pairs! {}),
            ("/bar/", Some("/bar/"), slice_pairs! {}),
            (
                "/bar/x",
                Some("/bar/*catchall"),
                slice_pairs! { "catchall" => "x" }
            ),
        ]
    );
}

#[test]
fn overlap() {
    test_tree!(
        slices![
            "/foo",
            "/bar",
            "/*bar",
            "/baz",
            "/baz/",
            "/baz/x",
            "/baz/:xxx",
            "/",
            "/xxx/*x",
            "/xxx/"
        ],
        [
            ("/foo", Some("/foo"), slice_pairs! {}),
            ("/bar", Some("/bar"), slice_pairs! {}),
            ("/baz", Some("/baz"), slice_pairs! {}),
            ("/baz/", Some("/baz/"), slice_pairs! {}),
            ("/baz/x", Some("/baz/x"), slice_pairs! {}),
            ("/???", Some("/*bar"), slice_pairs! { "bar" => "???" }),
            ("/", Some("/"), slice_pairs! {}),
            ("", None, slice_pairs! {}),
            ("/xxx/y", Some("/xxx/*x"), slice_pairs! { "x" => "y" }),
            ("/xxx/", Some("/xxx/"), slice_pairs! {}),
            ("/xxx", Some("/*bar"), slice_pairs! { "bar" => "xxx" }),
        ]
    );
}

#[test]
fn missing_trailing_slash_param() {
    test_tree!(
        slices!["/foo/:object/:id", "/foo/bar/baz", "/foo/secret/978/"],
        [
            (
                "/foo/secret/978/",
                Some("/foo/secret/978/"),
                slice_pairs! {}
            ),
            (
                "/foo/secret/978",
                Some("/foo/:object/:id"),
                slice_pairs! { "object" => "secret", "id" => "978" },
            ),
        ]
    );
}

#[test]
fn extra_trailing_slash_param() {
    test_tree!(
        slices!["/foo/:object/:id", "/foo/bar/baz", "/foo/secret/978"],
        [
            ("/foo/secret/978/", None, slice_pairs! {}),
            ("/foo/secret/978", Some("/foo/secret/978"), slice_pairs! {}),
        ]
    );
}

#[test]
fn missing_trailing_slash_catch_all() {
    test_tree!(
        slices!["/foo/*bar", "/foo/bar/baz", "/foo/secret/978/"],
        [
            (
                "/foo/secret/978",
                Some("/foo/*bar"),
                slice_pairs! { "bar" => "secret/978" },
            ),
            (
                "/foo/secret/978/",
                Some("/foo/secret/978/"),
                slice_pairs! {}
            ),
        ]
    );
}

#[test]
fn extra_trailing_slash_catch_all() {
    test_tree!(
        slices!["/foo/*bar", "/foo/bar/baz", "/foo/secret/978"],
        [
            (
                "/foo/secret/978/",
                Some("/foo/*bar"),
                slice_pairs! { "bar" => "secret/978/" },
            ),
            ("/foo/secret/978", Some("/foo/secret/978"), slice_pairs! {}),
        ]
    );
}

#[test]
fn double_overlap_trailing_slash() {
    test_tree!(
        slices![
            "/:object/:id",
            "/secret/:id/path",
            "/secret/978/",
            "/other/:object/:id/",
            "/other/an_object/:id",
            "/other/static/path",
            "/other/long/static/path/"
        ],
        [
            ("/secret/978/path/", None, slice_pairs! {}),
            ("/object/id/", None, slice_pairs! {}),
            ("/object/id/path", None, slice_pairs! {}),
            ("/other/object/1", None, slice_pairs! {}),
            ("/other/object/1/2", None, slice_pairs! {}),
            (
                "/other/an_object/1/",
                Some("/other/:object/:id/"),
                slice_pairs! { "object" => "an_object", "id" => "1" },
            ),
            (
                "/other/static/path/",
                Some("/other/:object/:id/"),
                slice_pairs! { "object" => "static", "id" => "path" },
            ),
            ("/other/long/static/path", None, slice_pairs! {}),
            ("/other/object/static/path", None, slice_pairs! {}),
        ]
    );
}

#[test]
fn trailing_slash_overlap() {
    test_tree!(
        slices!["/foo/:x/baz/", "/foo/:x/baz", "/foo/bar/bar"],
        [
            (
                "/foo/x/baz/",
                Some("/foo/:x/baz/"),
                slice_pairs! { "x" => "x" }
            ),
            (
                "/foo/x/baz",
                Some("/foo/:x/baz"),
                slice_pairs! { "x" => "x" }
            ),
            ("/foo/bar/bar", Some("/foo/bar/bar"), slice_pairs! {}),
        ]
    );
}

#[test]
fn trailing_slash() {
    test_tree!(
        slices![
            "/hi",
            "/b/",
            "/search/:query",
            "/cmd/:tool/",
            "/src/*filepath",
            "/x",
            "/x/y",
            "/y/",
            "/y/z",
            "/0/:id",
            "/0/:id/1",
            "/1/:id/",
            "/1/:id/2",
            "/aa",
            "/a/",
            "/admin",
            "/admin/static",
            "/admin/:category",
            "/admin/:category/:page",
            "/doc",
            "/doc/rust_faq.html",
            "/doc/rust1.26.html",
            "/no/a",
            "/no/b",
            "/no/a/b/*other",
            "/api/:page/:name",
            "/api/hello/:name/bar/",
            "/api/bar/:name",
            "/api/baz/foo",
            "/api/baz/foo/bar",
            "/foo/:p",
        ],
        [
            ("/hi/", None, slice_pairs! {}),
            ("/b", None, slice_pairs! {}),
            ("/search/rustacean/", None, slice_pairs! {}),
            ("/cmd/vet", None, slice_pairs! {}),
            ("/src", None, slice_pairs! {}),
            ("/src/", None, slice_pairs! {}),
            ("/x/", None, slice_pairs! {}),
            ("/y", None, slice_pairs! {}),
            ("/0/rust/", None, slice_pairs! {}),
            ("/1/rust", None, slice_pairs! {}),
            ("/a", None, slice_pairs! {}),
            ("/admin/", None, slice_pairs! {}),
            ("/doc/", None, slice_pairs! {}),
            ("/admin/static/", None, slice_pairs! {}),
            ("/admin/cfg/", None, slice_pairs! {}),
            ("/admin/cfg/users/", None, slice_pairs! {}),
            ("/api/hello/x/bar", None, slice_pairs! {}),
            ("/api/baz/foo/", None, slice_pairs! {}),
            ("/api/baz/bax/", None, slice_pairs! {}),
            ("/api/bar/huh/", None, slice_pairs! {}),
            ("/api/baz/foo/bar/", None, slice_pairs! {}),
            ("/api/world/abc/", None, slice_pairs! {}),
            ("/foo/pp/", None, slice_pairs! {}),
            ("/", None, slice_pairs! {}),
            ("/no", None, slice_pairs! {}),
            ("/no/", None, slice_pairs! {}),
            ("/no/a/b", None, slice_pairs! {}),
            ("/no/a/b/", None, slice_pairs! {}),
            ("/_", None, slice_pairs! {}),
            ("/_/", None, slice_pairs! {}),
            ("/api", None, slice_pairs! {}),
            ("/api/", None, slice_pairs! {}),
            ("/api/hello/x/foo", None, slice_pairs! {}),
            ("/api/baz/foo/bad", None, slice_pairs! {}),
            ("/foo/p/p", None, slice_pairs! {}),
        ]
    );
}

#[test]
fn backtracking_trailing_slash() {
    test_tree!(
        slices!["/a/:b/:c", "/a/b/:c/d/"],
        [("/a/b/c/d", None, slice_pairs! {}),]
    );
}

#[test]
fn root_trailing_slash() {
    test_tree!(
        slices!["/foo", "/bar", "/:baz"],
        [("/", None, slice_pairs! {}),]
    );
}

#[test]
fn catchall_overlap() {
    test_tree!(
        slices!["/yyy/*x", "/yyy*x"],
        [
            ("/yyy/y", Some("/yyy/*x"), slice_pairs! { "x" => "y" }),
            ("/yyy/", Some("/yyy*x"), slice_pairs! { "x" => "/" }),
        ]
    );
}

#[test]
fn basic() {
    test_tree!(
        slices![
            "/hi",
            "/contact",
            "/co",
            "/c",
            "/a",
            "/ab",
            "/doc/",
            "/doc/rust_faq.html",
            "/doc/rust1.26.html",
            "/ʯ",
            "/β",
            "/sd!here",
            "/sd$here",
            "/sd&here",
            "/sd'here",
            "/sd(here",
            "/sd)here",
            "/sd+here",
            "/sd,here",
            "/sd;here",
            "/sd=here",
        ],
        [
            ("/a", Some("/a"), slice_pairs! {}),
            ("/", None, slice_pairs! {}),
            ("/hi", Some("/hi"), slice_pairs! {}),
            ("/contact", Some("/contact"), slice_pairs! {}),
            ("/co", Some("/co"), slice_pairs! {}),
            ("/con", None, slice_pairs! {}),
            ("/cona", None, slice_pairs! {}),
            ("/no", None, slice_pairs! {}),
            ("/ab", Some("/ab"), slice_pairs! {}),
            ("/ʯ", Some("/ʯ"), slice_pairs! {}),
            ("/β", Some("/β"), slice_pairs! {}),
            ("/sd!here", Some("/sd!here"), slice_pairs! {}),
            ("/sd$here", Some("/sd$here"), slice_pairs! {}),
            ("/sd&here", Some("/sd&here"), slice_pairs! {}),
            ("/sd'here", Some("/sd'here"), slice_pairs! {}),
            ("/sd(here", Some("/sd(here"), slice_pairs! {}),
            ("/sd)here", Some("/sd)here"), slice_pairs! {}),
            ("/sd+here", Some("/sd+here"), slice_pairs! {}),
            ("/sd,here", Some("/sd,here"), slice_pairs! {}),
            ("/sd;here", Some("/sd;here"), slice_pairs! {}),
            ("/sd=here", Some("/sd=here"), slice_pairs! {}),
        ]
    );
}

#[test]
fn wildcard() {
    test_tree!(
        slices![
            "/",
            "/cmd/:tool/",
            "/cmd/:tool2/:sub",
            "/cmd/whoami",
            "/cmd/whoami/root",
            "/cmd/whoami/root/",
            "/src",
            "/src/",
            "/src/*filepath",
            "/search/",
            "/search/:query",
            "/search/actix-web",
            "/search/google",
            "/user_:name",
            "/user_:name/about",
            "/files/:dir/*filepath",
            "/doc/",
            "/doc/rust_faq.html",
            "/doc/rust1.26.html",
            "/info/:user/public",
            "/info/:user/project/:project",
            "/info/:user/project/rustlang",
            "/aa/*xx",
            "/ab/*xx",
            "/ab/hello*xx",
            "/:cc",
            "/c1/:dd/e",
            "/c1/:dd/e1",
            "/:cc/cc",
            "/:cc/:dd/ee",
            "/:cc/:dd/:ee/ff",
            "/:cc/:dd/:ee/:ff/gg",
            "/:cc/:dd/:ee/:ff/:gg/hh",
            "/get/test/abc/",
            "/get/:param/abc/",
            "/something/:paramname/thirdthing",
            "/something/secondthing/test",
            "/get/abc",
            "/get/:param",
            "/get/abc/123abc",
            "/get/abc/:param",
            "/get/abc/123abc/xxx8",
            "/get/abc/123abc/:param",
            "/get/abc/123abc/xxx8/1234",
            "/get/abc/123abc/xxx8/:param",
            "/get/abc/123abc/xxx8/1234/ffas",
            "/get/abc/123abc/xxx8/1234/:param",
            "/get/abc/123abc/xxx8/1234/kkdd/12c",
            "/get/abc/123abc/xxx8/1234/kkdd/:param",
            "/get/abc/:param/test",
            "/get/abc/123abd/:param",
            "/get/abc/123abddd/:param",
            "/get/abc/123/:param",
            "/get/abc/123abg/:param",
            "/get/abc/123abf/:param",
            "/get/abc/123abfff/:param",
        ],
        [
            ("/", Some("/"), slice_pairs! {}),
            ("/cmd/test", None, slice_pairs! {}),
            (
                "/cmd/test/",
                Some("/cmd/:tool/"),
                slice_pairs! { "tool" => "test" }
            ),
            (
                "/cmd/test/3",
                Some("/cmd/:tool2/:sub"),
                slice_pairs! { "tool2" => "test", "sub" => "3" }
            ),
            ("/cmd/who", None, slice_pairs! {}),
            (
                "/cmd/who/",
                Some("/cmd/:tool/"),
                slice_pairs! { "tool" => "who" }
            ),
            ("/cmd/whoami", Some("/cmd/whoami"), slice_pairs! {}),
            (
                "/cmd/whoami/",
                Some("/cmd/:tool/"),
                slice_pairs! { "tool" => "whoami" }
            ),
            (
                "/cmd/whoami/r",
                Some("/cmd/:tool2/:sub"),
                slice_pairs! { "tool2" => "whoami", "sub" => "r" }
            ),
            ("/cmd/whoami/r/", None, slice_pairs! {}),
            (
                "/cmd/whoami/root",
                Some("/cmd/whoami/root"),
                slice_pairs! {}
            ),
            (
                "/cmd/whoami/root/",
                Some("/cmd/whoami/root/"),
                slice_pairs! {}
            ),
            ("/src", Some("/src"), slice_pairs! {}),
            ("/src/", Some("/src/"), slice_pairs! {}),
            (
                "/src/some/file.png",
                Some("/src/*filepath"),
                slice_pairs! { "filepath" => "some/file.png" }
            ),
            ("/search/", Some("/search/"), slice_pairs! {}),
            (
                "/search/actix",
                Some("/search/:query"),
                slice_pairs! { "query" => "actix" }
            ),
            (
                "/search/actix-web",
                Some("/search/actix-web"),
                slice_pairs! {}
            ),
            (
                "/search/someth!ng+in+ünìcodé",
                Some("/search/:query"),
                slice_pairs! { "query" => "someth!ng+in+ünìcodé" }
            ),
            ("/search/someth!ng+in+ünìcodé/", None, slice_pairs! {}),
            (
                "/user_rustacean",
                Some("/user_:name"),
                slice_pairs! { "name" => "rustacean" }
            ),
            (
                "/user_rustacean/about",
                Some("/user_:name/about"),
                slice_pairs! { "name" => "rustacean" }
            ),
            (
                "/files/js/inc/framework.js",
                Some("/files/:dir/*filepath"),
                slice_pairs! { "dir" => "js", "filepath" => "inc/framework.js" }
            ),
            (
                "/info/gordon/public",
                Some("/info/:user/public"),
                slice_pairs! { "user" => "gordon" }
            ),
            (
                "/info/gordon/project/rust",
                Some("/info/:user/project/:project"),
                slice_pairs! { "user" => "gordon", "project" => "rust" }
            ),
            (
                "/info/gordon/project/rustlang",
                Some("/info/:user/project/rustlang"),
                slice_pairs! { "user" => "gordon" }
            ),
            ("/aa/", None, slice_pairs! {}),
            ("/aa/aa", Some("/aa/*xx"), slice_pairs! { "xx" => "aa" }),
            ("/ab/ab", Some("/ab/*xx"), slice_pairs! { "xx" => "ab" }),
            (
                "/ab/hello-world",
                Some("/ab/hello*xx"),
                slice_pairs! { "xx" => "-world" }
            ),
            ("/a", Some("/:cc"), slice_pairs! { "cc" => "a" }),
            ("/all", Some("/:cc"), slice_pairs! { "cc" => "all" }),
            ("/d", Some("/:cc"), slice_pairs! { "cc" => "d" }),
            ("/ad", Some("/:cc"), slice_pairs! { "cc" => "ad" }),
            ("/dd", Some("/:cc"), slice_pairs! { "cc" => "dd" }),
            ("/dddaa", Some("/:cc"), slice_pairs! { "cc" => "dddaa" }),
            ("/aa", Some("/:cc"), slice_pairs! { "cc" => "aa" }),
            ("/aaa", Some("/:cc"), slice_pairs! { "cc" => "aaa" }),
            ("/aaa/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "aaa" }),
            ("/ab", Some("/:cc"), slice_pairs! { "cc" => "ab" }),
            ("/abb", Some("/:cc"), slice_pairs! { "cc" => "abb" }),
            ("/abb/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "abb" }),
            ("/allxxxx", Some("/:cc"), slice_pairs! { "cc" => "allxxxx" }),
            ("/alldd", Some("/:cc"), slice_pairs! { "cc" => "alldd" }),
            ("/all/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "all" }),
            ("/a/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "a" }),
            ("/c1/d/e", Some("/c1/:dd/e"), slice_pairs! { "dd" => "d" }),
            ("/c1/d/e1", Some("/c1/:dd/e1"), slice_pairs! { "dd" => "d" }),
            (
                "/c1/d/ee",
                Some("/:cc/:dd/ee"),
                slice_pairs! { "cc" => "c1", "dd" => "d" }
            ),
            ("/cc/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "cc" }),
            ("/ccc/cc", Some("/:cc/cc"), slice_pairs! { "cc" => "ccc" }),
            (
                "/deedwjfs/cc",
                Some("/:cc/cc"),
                slice_pairs! { "cc" => "deedwjfs" }
            ),
            (
                "/acllcc/cc",
                Some("/:cc/cc"),
                slice_pairs! { "cc" => "acllcc" }
            ),
            ("/get/test/abc/", Some("/get/test/abc/"), slice_pairs! {}),
            (
                "/get/te/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "te" }
            ),
            (
                "/get/testaa/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "testaa" }
            ),
            (
                "/get/xx/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "xx" }
            ),
            (
                "/get/tt/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "tt" }
            ),
            (
                "/get/a/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "a" }
            ),
            (
                "/get/t/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "t" }
            ),
            (
                "/get/aa/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "aa" }
            ),
            (
                "/get/abas/abc/",
                Some("/get/:param/abc/"),
                slice_pairs! { "param" => "abas" }
            ),
            (
                "/something/secondthing/test",
                Some("/something/secondthing/test"),
                slice_pairs! {}
            ),
            (
                "/something/abcdad/thirdthing",
                Some("/something/:paramname/thirdthing"),
                slice_pairs! { "paramname" => "abcdad" }
            ),
            (
                "/something/secondthingaaaa/thirdthing",
                Some("/something/:paramname/thirdthing"),
                slice_pairs! { "paramname" => "secondthingaaaa" }
            ),
            (
                "/something/se/thirdthing",
                Some("/something/:paramname/thirdthing"),
                slice_pairs! { "paramname" => "se" }
            ),
            (
                "/something/s/thirdthing",
                Some("/something/:paramname/thirdthing"),
                slice_pairs! { "paramname" => "s" }
            ),
            (
                "/c/d/ee",
                Some("/:cc/:dd/ee"),
                slice_pairs! { "cc" => "c", "dd" => "d" }
            ),
            (
                "/c/d/e/ff",
                Some("/:cc/:dd/:ee/ff"),
                slice_pairs! { "cc" => "c", "dd" => "d", "ee" => "e" }
            ),
            (
                "/c/d/e/f/gg",
                Some("/:cc/:dd/:ee/:ff/gg"),
                slice_pairs! { "cc" => "c", "dd" => "d", "ee" => "e", "ff" => "f" }
            ),
            (
                "/c/d/e/f/g/hh",
                Some("/:cc/:dd/:ee/:ff/:gg/hh"),
                slice_pairs! { "cc" => "c", "dd" => "d", "ee" => "e", "ff" => "f", "gg" => "g" }
            ),
            (
                "/cc/dd/ee/ff/gg/hh",
                Some("/:cc/:dd/:ee/:ff/:gg/hh"),
                slice_pairs! { "cc" => "cc", "dd" => "dd", "ee" => "ee", "ff" => "ff", "gg" => "gg" }
            ),
            ("/get/abc", Some("/get/abc"), slice_pairs! {}),
            (
                "/get/a",
                Some("/get/:param"),
                slice_pairs! { "param" => "a" }
            ),
            (
                "/get/abz",
                Some("/get/:param"),
                slice_pairs! { "param" => "abz" }
            ),
            (
                "/get/12a",
                Some("/get/:param"),
                slice_pairs! { "param" => "12a" }
            ),
            (
                "/get/abcd",
                Some("/get/:param"),
                slice_pairs! { "param" => "abcd" }
            ),
            ("/get/abc/123abc", Some("/get/abc/123abc"), slice_pairs! {}),
            (
                "/get/abc/12",
                Some("/get/abc/:param"),
                slice_pairs! { "param" => "12" }
            ),
            (
                "/get/abc/123ab",
                Some("/get/abc/:param"),
                slice_pairs! { "param" => "123ab" }
            ),
            (
                "/get/abc/xyz",
                Some("/get/abc/:param"),
                slice_pairs! { "param" => "xyz" }
            ),
            (
                "/get/abc/123abcddxx",
                Some("/get/abc/:param"),
                slice_pairs! { "param" => "123abcddxx" }
            ),
            (
                "/get/abc/123abc/xxx8",
                Some("/get/abc/123abc/xxx8"),
                slice_pairs! {}
            ),
            (
                "/get/abc/123abc/x",
                Some("/get/abc/123abc/:param"),
                slice_pairs! { "param" => "x" }
            ),
            (
                "/get/abc/123abc/xxx",
                Some("/get/abc/123abc/:param"),
                slice_pairs! { "param" => "xxx" }
            ),
            (
                "/get/abc/123abc/abc",
                Some("/get/abc/123abc/:param"),
                slice_pairs! { "param" => "abc" }
            ),
            (
                "/get/abc/123abc/xxx8xxas",
                Some("/get/abc/123abc/:param"),
                slice_pairs! { "param" => "xxx8xxas" }
            ),
            (
                "/get/abc/123abc/xxx8/1234",
                Some("/get/abc/123abc/xxx8/1234"),
                slice_pairs! {}
            ),
            (
                "/get/abc/123abc/xxx8/1",
                Some("/get/abc/123abc/xxx8/:param"),
                slice_pairs! { "param" => "1" }
            ),
            (
                "/get/abc/123abc/xxx8/123",
                Some("/get/abc/123abc/xxx8/:param"),
                slice_pairs! { "param" => "123" }
            ),
            (
                "/get/abc/123abc/xxx8/78k",
                Some("/get/abc/123abc/xxx8/:param"),
                slice_pairs! { "param" => "78k" }
            ),
            (
                "/get/abc/123abc/xxx8/1234xxxd",
                Some("/get/abc/123abc/xxx8/:param"),
                slice_pairs! { "param" => "1234xxxd" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/ffas",
                Some("/get/abc/123abc/xxx8/1234/ffas"),
                slice_pairs! {}
            ),
            (
                "/get/abc/123abc/xxx8/1234/f",
                Some("/get/abc/123abc/xxx8/1234/:param"),
                slice_pairs! { "param" => "f" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/ffa",
                Some("/get/abc/123abc/xxx8/1234/:param"),
                slice_pairs! { "param" => "ffa" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kka",
                Some("/get/abc/123abc/xxx8/1234/:param"),
                slice_pairs! { "param" => "kka" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/ffas321",
                Some("/get/abc/123abc/xxx8/1234/:param"),
                slice_pairs! { "param" => "ffas321" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/12c",
                Some("/get/abc/123abc/xxx8/1234/kkdd/12c"),
                slice_pairs! {}
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/1",
                Some("/get/abc/123abc/xxx8/1234/kkdd/:param"),
                slice_pairs! { "param" => "1" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/12",
                Some("/get/abc/123abc/xxx8/1234/kkdd/:param"),
                slice_pairs! { "param" => "12" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/12b",
                Some("/get/abc/123abc/xxx8/1234/kkdd/:param"),
                slice_pairs! { "param" => "12b" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/34",
                Some("/get/abc/123abc/xxx8/1234/kkdd/:param"),
                slice_pairs! { "param" => "34" }
            ),
            (
                "/get/abc/123abc/xxx8/1234/kkdd/12c2e3",
                Some("/get/abc/123abc/xxx8/1234/kkdd/:param"),
                slice_pairs! { "param" => "12c2e3" }
            ),
            (
                "/get/abc/12/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "12" }
            ),
            (
                "/get/abc/123abdd/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123abdd" }
            ),
            (
                "/get/abc/123abdddf/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123abdddf" }
            ),
            (
                "/get/abc/123ab/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123ab" }
            ),
            (
                "/get/abc/123abgg/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123abgg" }
            ),
            (
                "/get/abc/123abff/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123abff" }
            ),
            (
                "/get/abc/123abffff/test",
                Some("/get/abc/:param/test"),
                slice_pairs! { "param" => "123abffff" }
            ),
            (
                "/get/abc/123abd/test",
                Some("/get/abc/123abd/:param"),
                slice_pairs! { "param" => "test" }
            ),
            (
                "/get/abc/123abddd/test",
                Some("/get/abc/123abddd/:param"),
                slice_pairs! { "param" => "test" }
            ),
            (
                "/get/abc/123/test22",
                Some("/get/abc/123/:param"),
                slice_pairs! { "param" => "test22" }
            ),
            (
                "/get/abc/123abg/test",
                Some("/get/abc/123abg/:param"),
                slice_pairs! { "param" => "test" }
            ),
            (
                "/get/abc/123abf/testss",
                Some("/get/abc/123abf/:param"),
                slice_pairs! { "param" => "testss" }
            ),
            (
                "/get/abc/123abfff/te",
                Some("/get/abc/123abfff/:param"),
                slice_pairs! { "param" => "te" }
            ),
        ]
    );
}
