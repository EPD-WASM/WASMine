(module
    (func $my_function (export "_start") (result i32)
        i32.const 10

        i32.const 1
        (if (result i32)

            (then
                i32.const 20
            )

            (else
                i32.const -1
                return
            )
        )
        i32.add

        (block (result i32)
            i32.const 5
            i32.const 7
            i32.add
            br 0
        )

        i32.add
        return
    )
)
