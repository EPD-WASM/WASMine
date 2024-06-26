(module
    (memory 1)
    (func $my_function (export "_start") (result i32)
        i32.const 1
        i32.const 42
        i32.store

        i32.const 1
        i32.load
        return
    )
)
