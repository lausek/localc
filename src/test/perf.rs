use super::*;

#[test]
fn fib() {
    use std::time::*;

    // we compare the runtime of popular languages to localc. the reference
    // values are taken at iteration 46.

    const PHP_TIME: Duration = Duration::from_millis(195468);
    const JULIA_TIME: Duration = Duration::from_millis(8530);
    const CPP_TIME: Duration = Duration::from_millis(4559);
    const TO_BEAT: Duration = PHP_TIME;

    let mut repl = Repl::new();

    repl.run("f(0) = 0").unwrap();
    repl.run("f(1) = 1").unwrap();
    repl.run("f(x) = f(x - 1) + f(x - 2)").unwrap();

    for i in 0..46 {
        let start = Instant::now();
        repl.run(&format!("f({})", i)).unwrap();
        let end = Instant::now();
        let time = end.duration_since(start);

        if TO_BEAT <= time {
            panic!(
                "opponent runtime faster at iteration {} (total time: {:?} ms)",
                i,
                time.as_millis()
            );
        }
    }
}
