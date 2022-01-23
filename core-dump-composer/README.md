# core-dump-composer

A process for collecting core dumps on kubernetes worker nodes.

## development

Development for this component is possible without a kubernetes environment.
The tests simulate a core dump by piping the `./mocks/test.core` to stdio with preset arguments and providing a bash script to act as a mock for `crictl`

Tests **MUST** be ran in single thread mode as the mock `crictl` is different for each test.

Tests **MUST** be ran from the specific project folder not the top level workspace.

```
cd core-dump-composer
cargo test -- --test-threads=1
```

For verbose logging set the LOG_LEVEL environment variable and the `composer.log` will be written into `../target/debug/composer.log`
```
LOG_LEVEL=Debug cargo test -- --test-threads=1
```

A failed test run will leave an output folder at `./core-dump-composer/output`.
This will need to be delete before each run as the files in this folder are counted.
