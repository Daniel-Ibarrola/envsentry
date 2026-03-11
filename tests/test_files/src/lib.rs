use std::env;

fn read_some_variables() {
    let (var, var2) = (
        std::env::var("MISSING_VAR_1").unwrap(), std::env::var("MISSING_VAR_2").unwrap()
    );
}