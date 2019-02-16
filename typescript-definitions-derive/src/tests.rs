#[cfg(test)]
mod macro_test {
    use super::quote;
    use super::Typescriptify;
    use insta::assert_snapshot_matches;
    #[test]
    // #[should_panic]
    fn tag_clash_in_enum() {
        let tokens = quote!(
            #[derive(Serialize)]
            #[serde(tag = "kind")]
            enum A {
                Unit,
                B { kind: i32, b: String },
            }
        );

        let result = std::panic::catch_unwind(move || Typescriptify::parse(true, tokens));
        match result {
            Ok(_x) => assert!(false, "expecting panic!"),
            Err(ref msg) => assert_snapshot_matches!( msg.downcast_ref::<String>().unwrap(),
            @r###"called `Result::unwrap()` on an `Err` value: "2 errors:\n\t# variant field name `kind` conflicts with internal tag\n\t# clash with field in \"A::B\". Maybe use a #[serde(content=\"...\")] attribute.""###
            ),
        }
    }
    #[test]
    fn flatten_is_fail() {
        let tokens = quote!(
            #[derive(Serialize)]
            struct SSS {
                a: i32,
                b: f64,
                #[serde(flatten)]
                c: DDD,
            }
        );
        let result = std::panic::catch_unwind(move || Typescriptify::parse(true, tokens));
        match result {
            Ok(_x) => assert!(false, "expecting panic!"),
            Err(ref msg) => assert_snapshot_matches!( msg.downcast_ref::<String>().unwrap(),
            @r###"called `Result::unwrap()` on an `Err` value: "SSS: #[serde(flatten)] does not work for typescript-definitions currently""###
            ),
        }
    }

    #[test]
    fn verify_is_recognized() {
        let tokens = quote!(
            #[derive(Serialize)]
            #[typescript(verify = "blah")]
            struct S {
                a: i32,
                b: f64,
            }
        );
        let result = std::panic::catch_unwind(move || Typescriptify::parse(true, tokens));
        match result {
            Ok(_x) => assert!(false, "expecting panic!"),
            Err(ref msg) => assert_snapshot_matches!( msg.downcast_ref::<String>().unwrap(),
            @r###"S: verify must be true or false not ""blah"""###
            ),
        }
    }
    #[test]
    fn turbo_fish() {
        let tokens = quote!(
            #[derive(TypeScriptify)]
            #[typescript(turbo_fish="<i32>")]
            struct S<T> {
                a: i32,
                b: Vec<T>,
            }
        );
        let ty = Typescriptify::parse(true, tokens);
        let i = &ty.ident;
        let g = ty.attrs.turbo_fish
                .map_or_else(|| quote!(), |s| s.parse::<proc_macro2::TokenStream>().unwrap());
        let res  = quote!(#i#g::type_scriptify()).to_string();
        assert_snapshot_matches!(res, 
        @"S < i32 > :: type_scriptify ( )" );

    }

}