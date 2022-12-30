mod amqp_connect_config;
mod amqp_wrapper;
mod api;
mod config;
mod error;
mod state;
mod storage;
mod tests;

#[cfg(test)]
#[test]
fn test_strs() {
    let array = vec!["abcd", "efgh"];
    let a = "abcd".to_string();

    if array.contains(&a.as_str()) {
        print!("OK")
    } else {
        print!("KO")
    }
}
