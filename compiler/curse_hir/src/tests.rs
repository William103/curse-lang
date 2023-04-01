use super::*;

const FIB: &str = r#"
let fib : i32 () -> i32 =
    |0| 0 else
    |1| 1 else
    |n| n - 1 fib () + (n - 2 fib ())

let main : () () -> () = ||
    10 fib () print ()
"#;

const TWICE: &str = r#"
let inc: i32 () -> i32 = |n|
    n + 1

let twice: (i32 () -> i32) i32 -> i32 = |f, x|
    x f () f ()

let main: () () -> () = ||
    inc twice 5 in print
"#;

const SUPERCHARGE: &str = r#"
let inc: i32 () -> i32 = |n|
    n + 1

// Given a function, return a function thats like it but calls the provided fn twice!
let supercharge: (i32 () -> i32) () -> i32 () -> i32 = |f|
    |x| x f () f ()

let main: () () -> () = ||
    5 (inc supercharge ()) () print ()
"#;

const INVALID: &str = r#"
let inc: i32 () -> i32 = |n|
    n + 1

// Given a function, return a function thats like it but calls the provided fn twice!
let supercharge: (i32 () -> i32) () -> i32 () -> i32 = |f|
    |x| x f () f ()

let main: () () -> () = ||
    // should be:
    // 5 in (inc supercharge ()) in print
    inc supercharge () in 5 in print
"#;

const ADDING: &str = r#"
let apply a b c: (a b -> c) (a, b) -> c = |f, (a, b)|
    a f b

let main: () () -> () = ||
    (+) apply (4, 5) in print
"#;

const IN2: &str = r#"
let in2 a b: a (a () -> b) -> b = |x, f|
    x f ()

let main: () () -> () = ||
    5 in2 print
"#;

#[test]
fn test_branching_typeck() {
    let program = ADDING;

    let ctx = curse_parse::Context::new();
    let program = curse_parse::parse_program(&ctx, program).unwrap();

    let mut allocations = Allocations::default();
    let mut env = Env::new(&mut allocations);

    // temporary for now until we can have custom named types
    let type_scope = HashMap::new();

    let globals = env
        .default_globals()
        .chain(program.items.iter().map(|item| {
            // Since items (e.g. functions for now) can be generic over types,
            // we need to extend the set of currently in-scope types with the
            // generics that this item introduces. To avoid bringing the types
            // into the global program scope, we'll create a temporary inner scope
            let mut inner_type_scope = type_scope.clone();

            let mut typevars = Vec::with_capacity(item.generics.len());

            for generic in item.generics.iter() {
                let (var, ty) = env.new_typevar();
                typevars.push(var);
                inner_type_scope.insert(generic.literal, ty);
            }

            (
                item.name.literal.to_string(),
                Polytype {
                    typevars,
                    typ: env.type_from_ast(item.typ, &inner_type_scope),
                },
            )
        }))
        .collect();

    let mut locals = Vec::with_capacity(16);
    let mut bindings = Bindings::new(&globals, &mut locals);

    let mut errors = vec![];

    let main = program
        .items
        .iter()
        .find(|item| item.name.literal == "main")
        .unwrap()
        .expr;

    // Right now this won't work for finding the types of generic functions.
    // We need some way to preserve their scope.
    let result = env.lower(&mut bindings, main, &type_scope, &mut errors);
    if errors.is_empty() {
        let _expr = result.expect("no errors");
    } else {
        println!("failed");
    }
    // Put the result into: https://edotor.net/
    println!("{}", env.equations);

    // println!("{:?}", env.typevars.borrow());
    // println!("{t:#?}");
}

const PROG2: &str = r#"
    0 range 10
        map (|x| x + 1)
        filter (|x| x < 4)
        collect ()
        in |vec|
    
    vec print ()
"#;
