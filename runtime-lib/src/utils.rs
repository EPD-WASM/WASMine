macro_rules! macro_invoke_for_each_function_signature {
    ($macro:ident) => {
        $macro!(0);
        $macro!(1 Param1);
        $macro!(2 Param1 Param2);
        $macro!(3 Param1 Param2 Param3);
        $macro!(4 Param1 Param2 Param3 Param4);
        $macro!(5 Param1 Param2 Param3 Param4 Param5);
        $macro!(6 Param1 Param2 Param3 Param4 Param5 Param6);
        $macro!(7 Param1 Param2 Param3 Param4 Param5 Param6 Param7);
        $macro!(8 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8);
        $macro!(9 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9);
        $macro!(10 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10);
        $macro!(11 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11);
        $macro!(12 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12);
        $macro!(13 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13);
        $macro!(14 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14);
        $macro!(15 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14 Param15);
        $macro!(16 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14 Param15 Param16);
    };
}
pub(crate) use macro_invoke_for_each_function_signature;

#[derive(Clone)]
pub(crate) enum Either<T1, T2> {
    Left(T1),
    Right(T2),
}
