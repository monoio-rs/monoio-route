//! The test cases are mainly converted from matchit's test.

macro_rules! make_tree {
    () => {
        let mut tree = monoio_route::Tree::new();
        #[allow(unused)]
        macro_rules! ok {
            ($path:expr) => {
                assert!(tree.insert($path, ()).is_ok());
            };
        }
        #[allow(unused)]
        macro_rules! err {
            ($path:expr) => {
                assert!(tree.insert($path, ()).is_err());
            };
        }
    };
}

#[test]
fn wildcard_conflict() {
    make_tree!();
    ok!(b"/cmd/:tool/:sub");
    ok!(b"/foo/bar");
    ok!(b"/foo/:name");
    err!(b"/foo/:names");
    ok!(b"/cmd/*path");
    ok!(b"/cmd/:xxx/names");
    ok!(b"/cmd/:tool/:xxx/foo");
    ok!(b"/src/*filepath");
    ok!(b"/src/:file");
    ok!(b"/src/static.json");
    ok!(b"/src/$filepathx");
    ok!(b"/src/");
    ok!(b"/src/foo/bar");
    ok!(b"/src1/");
    ok!(b"/src1/*filepath");
    ok!(b"/src2*filepath");
    ok!(b"/src2/*filepath");
    ok!(b"/src2/");
    ok!(b"/src2");
    ok!(b"/src3");
    ok!(b"/src3/*filepath");
    ok!(b"/search/:query");
    ok!(b"/search/valid");
    ok!(b"/user_:name");
    ok!(b"/user_x");
    err!(b"/user_:bar");
    ok!(b"/id:id");
    ok!(b"/id/:id");
}

#[test]
fn invalid_catchall() {
    make_tree!();
    ok!(b"/non-leading-*catchall");
    ok!(b"/foo/bar*catchall");
    err!(b"/src/*filepath/x");
    ok!(b"/src2/");
    err!(b"/src2/*filepath/x");
}

#[test]
fn catchall_root_conflict() {
    make_tree!();
    ok!(b"/");
    ok!(b"/*filepath");
}

#[test]
fn child_conflict() {
    make_tree!();
    ok!(b"/cmd/vet");
    ok!(b"/cmd/:tool");
    ok!(b"/cmd/:tool/:sub");
    ok!(b"/cmd/:tool/misc");
    err!(b"/cmd/:tool/:bad");
    ok!(b"/src/AUTHORS");
    ok!(b"/src/*filepath");
    ok!(b"/user_x");
    ok!(b"/user_:name");
    ok!(b"/id/:id");
    ok!(b"/id:id");
    ok!(b"/:id");
    ok!(b"/*filepath");
}

#[test]
fn duplicates() {
    make_tree!();
    ok!(b"/");
    err!(b"/");
    ok!(b"/doc/");
    err!(b"/doc/");
    ok!(b"/src/*filepath");
    err!(b"/src/*filepath");
    ok!(b"/search/:query");
    err!(b"/search/:query");
    ok!(b"/user_:name");
    err!(b"/user_:name");
}

#[test]
fn unnamed_param() {
    make_tree!();
    err!(b"/:");
    err!(b"/user:/");
    err!(b"/cmd/:/");
    err!(b"/src/*");
}

#[test]
fn double_params() {
    make_tree!();
    err!(b"/::");
    err!(b"/::/");
    err!(b"/:*/*/");
}

#[test]
fn normalized_conflict() {
    make_tree!();
    ok!(b"/x/:foo/bar");
    err!(b"/x/:bar/bar");
    ok!(b"/:y/bar/baz");
    ok!(b"/:y/baz/baz");
    ok!(b"/:z/bar/bat");
    err!(b"/:z/bar/baz");
}

#[test]
fn more_conflicts() {
    make_tree!();
    ok!(b"/con:tact");
    ok!(b"/who/are/*you");
    ok!(b"/who/foo/hello");
    ok!(b"/whose/:users/:name");
    ok!(b"/who/are/foo");
    ok!(b"/who/are/foo/bar");
    err!(b"/con:nection");
    err!(b"/whose/:users/:user");
}

#[test]
fn catchall_static_overlap() {
    {
        make_tree!();
        ok!(b"/bar");
        ok!(b"/bar/");
        ok!(b"/bar/*foo");
    }
    {
        make_tree!();
        ok!(b"/foo");
        ok!(b"/*bar");
        ok!(b"/bar");
        ok!(b"/baz");
        ok!(b"/baz/:split");
        ok!(b"/");
        err!(b"/*bar");
        err!(b"/*zzz");
        ok!(b"/:xxx");
    }
    {
        make_tree!();
        ok!(b"/*bar");
        ok!(b"/bar");
        ok!(b"/bar/x");
        ok!(b"/bar_:x");
        err!(b"/bar_:x");
        ok!(b"/bar_:x/y");
        ok!(b"/bar/:x");
    }
}

#[test]
fn duplicate_conflict() {
    make_tree!();
    ok!(b"/hey");
    ok!(b"/hey/users");
    ok!(b"/hey/user");
    err!(b"/hey/user");
}

#[test]
fn bare_catchall() {
    make_tree!();
    ok!(b"/*foo");
    ok!(b"foo/*bar");
}
