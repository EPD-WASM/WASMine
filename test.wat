(module
    ;; add(a, b) returns a+b
    (func $add (export "add") (param $a i32) (param $b i32)
        unreachable
        (block $my_block
            (block $my_block2
                (i32.add (local.get $a) (local.get $b))
                drop
                br $my_block
                unreachable
            )
        )
    )
)
