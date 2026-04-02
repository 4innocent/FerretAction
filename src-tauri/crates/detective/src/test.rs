pub fn test_fn() {
    println!(
        "testFn called{}",
        std::env::current_dir().unwrap().display()
    );
}
