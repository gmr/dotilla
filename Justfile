test:
    @cargo clippy --all-targets -- -D warnings
    @cargo llvm-cov --html --color auto --no-cfg-coverage
    @cargo llvm-cov report --summary-only
