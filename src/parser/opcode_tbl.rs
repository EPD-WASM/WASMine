use self::storage::InstructionEncoder;
use self::wasm_stream_reader::WasmStreamReader;
use super::*;
use crate::instructions::*;
use crate::parser::error::ParserError;

/// Reference: https://webassembly.github.io/spec/core/bikeshed/#a7-index-of-instructions
#[rustfmt::skip]
pub(crate) const LVL1_JMP_TABLE: [fn(&mut Context, &mut WasmStreamReader, &mut InstructionEncoder) -> ParseResult; 256] = [
    /* Control Instructions */
    /* instr ::= 0x00 ⇒ unreachable
            | 0x01 ⇒ nop
            | 0x02 bt:blocktype (in:instr)* 0x0B ⇒ block bt in* end
            | 0x03 bt:blocktype (in:instr)* 0x0B ⇒ loop bt in* end
            | 0x04 bt:blocktype (in:instr)* 0x0B ⇒ if bt in* else 𝜖 end
            | 0x04 bt:blocktype (in1:instr)* 0x05 (in2:instr)* 0x0B ⇒ if bt in* 1 else in* 2 end
            | 0x0C 𝑙:labelidx ⇒ br 𝑙
            | 0x0D 𝑙:labelidx ⇒ br_if 𝑙
            | 0x0E 𝑙*:vec(labelidx) 𝑙𝑁 :labelidx ⇒ br_table 𝑙* 𝑙𝑁
            | 0x0F ⇒ return
            | 0x10 𝑥:funcidx ⇒ call 𝑥
            | 0x11 𝑦:typeidx 𝑥:tableidx ⇒ call_indirect 𝑥 𝑦 */
    /* 0x00 */ unreachable,
    /* 0x01 */ nop,
    /* 0x02 */ block,
    /* 0x03 */ r#loop,
    /* 0x04 */ if_else,

    /* 0x05 */ r#else,

    /* Unused */
    /* 0x06 - 0x0A */ e, e, e, e, e,

    /* 0x0B */ end,

    /* 0x0C */ br,
    /* 0x0D */ br_if,
    /* 0x0E */ br_table,
    /* 0x0F */ r#return,
    /* 0x10 */ call,
    /* 0x11 */ call_indirect,

    /* Unused */
    /* 0x12 - 0x19 */ e, e, e, e, e, e, e, e,

    /* Parametric Instructions */
    /* instr ::= ...
                | 0x1A ⇒ drop
                | 0x1B ⇒ select
                | 0x1C 𝑡*:vec(valtype) ⇒ select 𝑡* */
    /* 0x1A */ drop,
    /* 0x1B */ select_numeric,
    /* 0x1C */ select_generic,

    /* Unused */
    /* 0x1D - 0x1F */ e, e, e,

    /* Variable Instructions */
    /* instr ::= ...
                | 0x20 𝑥:localidx ⇒ local.get 𝑥
                | 0x21 𝑥:localidx ⇒ local.set 𝑥
                | 0x22 𝑥:localidx ⇒ local.tee 𝑥
                | 0x23 𝑥:globalidx ⇒ global.get 𝑥
                | 0x24 𝑥:globalidx ⇒ global.set 𝑥 */
    /* 0x20 */ local_get,
    /* 0x21 */ local_set,
    /* 0x22 */ local_tee,
    /* 0x23 */ global_get,
    /* 0x24 */ global_set,

    /* Table Instructions */
    /* instr ::= ...
                | 0x25 𝑥:tableidx ⇒ table.get 𝑥
                | 0x26 𝑥:tableidx ⇒ table.set 𝑥 */
    /* 0x25 */ table_get,
    /* 0x26 */ table_set,

    /* Unused */
    /* 0x27 */ e,

    /* Memory Instructions */
    /* instr ::= ...
                | 0x28 𝑚:memarg ⇒ i32.load 𝑚
                | 0x29 𝑚:memarg ⇒ i64.load 𝑚
                | 0x2A 𝑚:memarg ⇒ f32.load 𝑚
                | 0x2B 𝑚:memarg ⇒ f64.load 𝑚
                | 0x2C 𝑚:memarg ⇒ i32.load8_s 𝑚
                | 0x2D 𝑚:memarg ⇒ i32.load8_u 𝑚
                | 0x2E 𝑚:memarg ⇒ i32.load16_s 𝑚
                | 0x2F 𝑚:memarg ⇒ i32.load16_u 𝑚
                | 0x30 𝑚:memarg ⇒ i64.load8_s 𝑚
                | 0x31 𝑚:memarg ⇒ i64.load8_u 𝑚
                | 0x32 𝑚:memarg ⇒ i64.load16_s 𝑚
                | 0x33 𝑚:memarg ⇒ i64.load16_u 𝑚
                | 0x34 𝑚:memarg ⇒ i64.load32_s 𝑚
                | 0x35 𝑚:memarg ⇒ i64.load32_u 𝑚
                | 0x36 𝑚:memarg ⇒ i32.store 𝑚
                | 0x37 𝑚:memarg ⇒ i64.store 𝑚
                | 0x38 𝑚:memarg ⇒ f32.store 𝑚
                | 0x39 𝑚:memarg ⇒ f64.store 𝑚
                | 0x3A 𝑚:memarg ⇒ i32.store8 𝑚
                | 0x3B 𝑚:memarg ⇒ i32.store16 𝑚
                | 0x3C 𝑚:memarg ⇒ i64.store8 𝑚
                | 0x3D 𝑚:memarg ⇒ i64.store16 𝑚
                | 0x3E 𝑚:memarg ⇒ i64.store32 𝑚
                | 0x3F 0x00 ⇒ memory.size
                | 0x40 0x00 ⇒ memory.grow
                | 0xFC 8:u32 𝑥:dataidx 0x00 ⇒ memory.init 𝑥
                | 0xFC 9:u32 𝑥:dataidx ⇒ data.drop 𝑥
                | 0xFC 10:u32 0x00 0x00 ⇒ memory.copy
                | 0xFC 11:u32 0x00 ⇒ memory.fill */
    /* 0x28 */ i32_load,
    /* 0x29 */ i64_load,
    /* 0x2A */ f32_load,
    /* 0x2B */ f64_load,
    /* 0x2C */ i32_load8_s,
    /* 0x2D */ i32_load8_u,
    /* 0x2E */ i32_load16_s,
    /* 0x2F */ i32_load16_u,
    /* 0x30 */ i64_load8_s,
    /* 0x31 */ i64_load8_u,
    /* 0x32 */ i64_load16_s,
    /* 0x33 */ i64_load16_u,
    /* 0x34 */ i64_load32_s,
    /* 0x35 */ i64_load32_u,
    /* 0x36 */ i32_store,
    /* 0x37 */ i64_store,
    /* 0x38 */ f32_store,
    /* 0x39 */ f64_store,
    /* 0x3A */ i32_store8,
    /* 0x3B */ i32_store16,
    /* 0x3C */ i64_store8,
    /* 0x3D */ i64_store16,
    /* 0x3E */ i64_store32,
    /* 0x3F */ memory_size,
    /* 0x40 */ memory_grow,

    /* Numeric Instructions */
    /* instr ::= ...
                | 0x41 𝑛:i32 ⇒ i32.const 𝑛
                | 0x42 𝑛:i64 ⇒ i64.const 𝑛
                | 0x43 𝑧:f32 ⇒ f32.const 𝑧
                | 0x44 𝑧:f64 ⇒ f64.const 𝑧
                | 0x45 ⇒ i32.eqz
                | 0x46 ⇒ i32.eq
                | 0x47 ⇒ i32.ne
                | 0x48 ⇒ i32.lt_s
                | 0x49 ⇒ i32.lt_u
                | 0x4A ⇒ i32.gt_s
                | 0x4B ⇒ i32.gt_u
                | 0x4C ⇒ i32.le_s
                | 0x4D ⇒ i32.le_u
                | 0x4E ⇒ i32.ge_s
                | 0x4F ⇒ i32.ge_u
                | 0x50 ⇒ i64.eqz
                | 0x51 ⇒ i64.eq
                | 0x52 ⇒ i64.ne
                | 0x53 ⇒ i64.lt_s
                | 0x54 ⇒ i64.lt_u
                | 0x55 ⇒ i64.gt_s
                | 0x56 ⇒ i64.gt_u
                | 0x57 ⇒ i64.le_s
                | 0x58 ⇒ i64.le_u
                | 0x59 ⇒ i64.ge_s
                | 0x5A ⇒ i64.ge_u
                | 0x5B ⇒ f32.eq
                | 0x5C ⇒ f32.ne
                | 0x5D ⇒ f32.lt
                | 0x5E ⇒ f32.gt
                | 0x5F ⇒ f32.le
                | 0x60 ⇒ f32.ge
                | 0x61 ⇒ f64.eq
                | 0x62 ⇒ f64.ne
                | 0x63 ⇒ f64.lt
                | 0x64 ⇒ f64.gt
                | 0x65 ⇒ f64.le
                | 0x66 ⇒ f64.ge
                | 0x67 ⇒ i32.clz
                | 0x68 ⇒ i32.ctz
                | 0x69 ⇒ i32.popcnt
                | 0x6A ⇒ i32.add
                | 0x6B ⇒ i32.sub
                | 0x6C ⇒ i32.mul
                | 0x6D ⇒ i32.div_s
                | 0x6E ⇒ i32.div_u
                | 0x6F ⇒ i32.rem_s
                | 0x70 ⇒ i32.rem_u
                | 0x71 ⇒ i32.and
                | 0x72 ⇒ i32.or
                | 0x73 ⇒ i32.xor
                | 0x74 ⇒ i32.shl
                | 0x75 ⇒ i32.shr_s
                | 0x76 ⇒ i32.shr_u
                | 0x77 ⇒ i32.rotl
                | 0x78 ⇒ i32.rotr
                | 0x79 ⇒ i64.clz
                | 0x7A ⇒ i64.ctz
                | 0x7B ⇒ i64.popcnt
                | 0x7C ⇒ i64.add
                | 0x7D ⇒ i64.sub
                | 0x7E ⇒ i64.mul
                | 0x7F ⇒ i64.div_s
                | 0x80 ⇒ i64.div_u
                | 0x81 ⇒ i64.rem_s
                | 0x82 ⇒ i64.rem_u
                | 0x83 ⇒ i64.and
                | 0x84 ⇒ i64.or
                | 0x85 ⇒ i64.xor
                | 0x86 ⇒ i64.shl
                | 0x87 ⇒ i64.shr_s
                | 0x88 ⇒ i64.shr_u
                | 0x89 ⇒ i64.rotl
                | 0x8A ⇒ i64.rotr
                | 0x8B ⇒ f32.abs
                | 0x8C ⇒ f32.neg
                | 0x8D ⇒ f32.ceil
                | 0x8E ⇒ f32.floor
                | 0x8F ⇒ f32.trunc
                | 0x90 ⇒ f32.nearest
                | 0x91 ⇒ f32.sqrt
                | 0x92 ⇒ f32.add
                | 0x93 ⇒ f32.sub
                | 0x94 ⇒ f32.mul
                | 0x95 ⇒ f32.div
                | 0x96 ⇒ f32.min
                | 0x97 ⇒ f32.max
                | 0x98 ⇒ f32.copysign
                | 0x99 ⇒ f64.abs
                | 0x9A ⇒ f64.neg
                | 0x9B ⇒ f64.ceil
                | 0x9C ⇒ f64.floor
                | 0x9D ⇒ f64.trunc
                | 0x9E ⇒ f64.nearest
                | 0x9F ⇒ f64.sqrt
                | 0xA0 ⇒ f64.add
                | 0xA1 ⇒ f64.sub
                | 0xA2 ⇒ f64.mul
                | 0xA3 ⇒ f64.div
                | 0xA4 ⇒ f64.min
                | 0xA5 ⇒ f64.max
                | 0xA6 ⇒ f64.copysign
                | 0xA7 ⇒ i32.wrap_i64
                | 0xA8 ⇒ i32.trunc_f32_s
                | 0xA9 ⇒ i32.trunc_f32_u
                | 0xAA ⇒ i32.trunc_f64_s
                | 0xAB ⇒ i32.trunc_f64_u
                | 0xAC ⇒ i64.extend_i32_s
                | 0xAD ⇒ i64.extend_i32_u
                | 0xAE ⇒ i64.trunc_f32_s
                | 0xAF ⇒ i64.trunc_f32_u
                | 0xB0 ⇒ i64.trunc_f64_s
                | 0xB1 ⇒ i64.trunc_f64_u
                | 0xB2 ⇒ f32.convert_i32_s
                | 0xB3 ⇒ f32.convert_i32_u
                | 0xB4 ⇒ f32.convert_i64_s
                | 0xB5 ⇒ f32.convert_i64_u
                | 0xB6 ⇒ f32.demote_f64
                | 0xB7 ⇒ f64.convert_i32_s
                | 0xB8 ⇒ f64.convert_i32_u
                | 0xB9 ⇒ f64.convert_i64_s
                | 0xBA ⇒ f64.convert_i64_u
                | 0xBB ⇒ f64.promote_f32
                | 0xBC ⇒ i32.reinterpret_f32
                | 0xBD ⇒ i64.reinterpret_f64
                | 0xBE ⇒ f32.reinterpret_i32
                | 0xBF ⇒ f64.reinterpret_i64

                | 0xC0 ⇒ i32.extend8_s
                | 0xC1 ⇒ i32.extend16_s
                | 0xC2 ⇒ i64.extend8_s
                | 0xC3 ⇒ i64.extend16_s
                | 0xC4 ⇒ i64.extend32_s */
    /* 0x41 */ i32_const_i32,
    /* 0x42 */ i64_const_i64,
    /* 0x43 */ f32_const_f32,
    /* 0x44 */ f64_const_f64,
    /* 0x45 */ i32_eqz,
    /* 0x46 */ i32_eq,
    /* 0x47 */ i32_ne,
    /* 0x48 */ i32_lt_s,
    /* 0x49 */ i32_lt_u,
    /* 0x4A */ i32_gt_s,
    /* 0x4B */ i32_gt_u,
    /* 0x4C */ i32_le_s,
    /* 0x4D */ i32_le_u,
    /* 0x4E */ i32_ge_s,
    /* 0x4F */ i32_ge_u,
    /* 0x50 */ i64_eqz,
    /* 0x51 */ i64_eq,
    /* 0x52 */ i64_ne,
    /* 0x53 */ i64_lt_s,
    /* 0x54 */ i64_lt_u,
    /* 0x55 */ i64_gt_s,
    /* 0x56 */ i64_gt_u,
    /* 0x57 */ i64_le_s,
    /* 0x58 */ i64_le_u,
    /* 0x59 */ i64_ge_s,
    /* 0x5A */ i64_ge_u,
    /* 0x5B */ f32_eq,
    /* 0x5C */ f32_ne,
    /* 0x5D */ f32_lt,
    /* 0x5E */ f32_gt,
    /* 0x5F */ f32_le,
    /* 0x60 */ f32_ge,
    /* 0x61 */ f64_eq,
    /* 0x62 */ f64_ne,
    /* 0x63 */ f64_lt,
    /* 0x64 */ f64_gt,
    /* 0x65 */ f64_le,
    /* 0x66 */ f64_ge,
    /* 0x67 */ i32_clz,
    /* 0x68 */ i32_ctz,
    /* 0x69 */ i32_popcnt,
    /* 0x6A */ i32_add,
    /* 0x6B */ i32_sub,
    /* 0x6C */ i32_mul,
    /* 0x6D */ i32_div_s,
    /* 0x6E */ i32_div_u,
    /* 0x6F */ i32_rem_s,
    /* 0x70 */ i32_rem_u,
    /* 0x71 */ i32_and,
    /* 0x72 */ i32_or,
    /* 0x73 */ i32_xor,
    /* 0x74 */ i32_shl,
    /* 0x75 */ i32_shr_s,
    /* 0x76 */ i32_shr_u,
    /* 0x77 */ i32_rotl,
    /* 0x78 */ i32_rotr,
    /* 0x79 */ i64_clz,
    /* 0x7A */ i64_ctz,
    /* 0x7B */ i64_popcnt,
    /* 0x7C */ i64_add,
    /* 0x7D */ i64_sub,
    /* 0x7E */ i64_mul,
    /* 0x7F */ i64_div_s,
    /* 0x80 */ i64_div_u,
    /* 0x81 */ i64_rem_s,
    /* 0x82 */ i64_rem_u,
    /* 0x83 */ i64_and,
    /* 0x84 */ i64_or,
    /* 0x85 */ i64_xor,
    /* 0x86 */ i64_shl,
    /* 0x87 */ i64_shr_s,
    /* 0x88 */ i64_shr_u,
    /* 0x89 */ i64_rotl,
    /* 0x8A */ i64_rotr,
    /* 0x8B */ f32_abs,
    /* 0x8C */ f32_neg,
    /* 0x8D */ f32_ceil,
    /* 0x8E */ f32_floor,
    /* 0x8F */ f32_trunc,
    /* 0x90 */ f32_nearest,
    /* 0x91 */ f32_sqrt,
    /* 0x92 */ f32_add,
    /* 0x93 */ f32_sub,
    /* 0x94 */ f32_mul,
    /* 0x95 */ f32_div,
    /* 0x96 */ f32_min,
    /* 0x97 */ f32_max,
    /* 0x98 */ f32_copysign,
    /* 0x99 */ f64_abs,
    /* 0x9A */ f64_neg,
    /* 0x9B */ f64_ceil,
    /* 0x9C */ f64_floor,
    /* 0x9D */ f64_trunc,
    /* 0x9E */ f64_nearest,
    /* 0x9F */ f64_sqrt,
    /* 0xA0 */ f64_add,
    /* 0xA1 */ f64_sub,
    /* 0xA2 */ f64_mul,
    /* 0xA3 */ f64_div,
    /* 0xA4 */ f64_min,
    /* 0xA5 */ f64_max,
    /* 0xA6 */ f64_copysign,
    /* 0xA7 */ i32_wrap_i64,
    /* 0xA8 */ i32_trunc_f32_s,
    /* 0xA9 */ i32_trunc_f32_u,
    /* 0xAA */ i32_trunc_f64_s,
    /* 0xAB */ i32_trunc_f64_u,
    /* 0xAC */ i64_extend_i32_s,
    /* 0xAD */ i64_extend_i32_u,
    /* 0xAE */ i64_trunc_f32_s,
    /* 0xAF */ i64_trunc_f32_u,
    /* 0xB0 */ i64_trunc_f64_s,
    /* 0xB1 */ i64_trunc_f64_u,
    /* 0xB2 */ f32_convert_i32_s,
    /* 0xB3 */ f32_convert_i32_u,
    /* 0xB4 */ f32_convert_i64_s,
    /* 0xB5 */ f32_convert_i64_u,
    /* 0xB6 */ f32_demote_f64,
    /* 0xB7 */ f64_convert_i32_s,
    /* 0xB8 */ f64_convert_i32_u,
    /* 0xB9 */ f64_convert_i64_s,
    /* 0xBA */ f64_convert_i64_u,
    /* 0xBB */ f64_promote_f32,
    /* 0xBC */ i32_reinterpret_f32,
    /* 0xBD */ i64_reinterpret_f64,
    /* 0xBE */ f32_reinterpret_i32,
    /* 0xBF */ f64_reinterpret_i64,
    /* 0xC0 */ i32_extend8_s,
    /* 0xC1 */ i32_extend16_s,
    /* 0xC2 */ i64_extend8_s,
    /* 0xC3 */ i64_extend16_s,
    /* 0xC4 */ i64_extend32_s,

    /* Unused */ e, e, e, e, e, e, e, e, e, e, e,

    /* 0xD0 */ ref_null,
    /* 0xD1 */ ref_is_null,
    /* 0xD2 */ ref_func,

    /* Unused */ e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e, e,

    /* 0xFC */ lvl2_instruction_gp,

    /* Vector Instructions */
    /*  instr ::= ...
                | 0xFD 0:u32 𝑚:memarg ⇒ v128.load 𝑚
                | 0xFD 1:u32 𝑚:memarg ⇒ v128.load8x8_s 𝑚
                | 0xFD 2:u32 𝑚:memarg ⇒ v128.load8x8_u 𝑚

            + LOTS MORE, all opcode 0xFD! */

    /* 0xFD */ lvl2_instruction_vec,

    /* Unused */ e, e
];

#[rustfmt::skip]
#[allow(non_upper_case_globals)]
pub(crate) const LVL2_JMP_TABLE_0xFC: [fn(&mut Context, &mut WasmStreamReader, &mut InstructionEncoder) -> ParseResult; 18] = [
    /* Table Instructions */
    /* instr ::= ...
                | 0xFC 12:u32 𝑦:elemidx 𝑥:tableidx ⇒ table.init 𝑥 𝑦
                | 0xFC 13:u32 𝑥:elemidx ⇒ elem.drop 𝑥
                | 0xFC 14:u32 𝑥:tableidx 𝑦:tableidx ⇒ table.copy 𝑥 𝑦
                | 0xFC 15:u32 𝑥:tableidx ⇒ table.grow 𝑥
                | 0xFC 16:u32 𝑥:tableidx ⇒ table.size 𝑥
                | 0xFC 17:u32 𝑥:tableidx ⇒ table.fill 𝑥
        AND

    Memory Instructions
        instr ::= ...
                | 0xFC 8:u32 𝑥:dataidx 0x00 ⇒ memory.init 𝑥
                | 0xFC 9:u32 𝑥:dataidx ⇒ data.drop 𝑥
                | 0xFC 10:u32 0x00 0x00 ⇒ memory.copy
                | 0xFC 11:u32 0x00 ⇒ memory.fill

        AND
    Numeric Instructions c'tned:
      instr ::= ...
            | 0xFC 0:u32 ⇒ i32.trunc_sat_f32_s
            | 0xFC 1:u32 ⇒ i32.trunc_sat_f32_u
            | 0xFC 2:u32 ⇒ i32.trunc_sat_f64_s
            | 0xFC 3:u32 ⇒ i32.trunc_sat_f64_u
            | 0xFC 4:u32 ⇒ i64.trunc_sat_f32_s
            | 0xFC 5:u32 ⇒ i64.trunc_sat_f32_u
            | 0xFC 6:u32 ⇒ i64.trunc_sat_f64_s
            | 0xFC 7:u32 ⇒ i64.trunc_sat_f64_u */

    /* 0 */ i32_trunc_sat_f32_s,
    /* 1 */ i32_trunc_sat_f32_u,
    /* 2 */ i32_trunc_sat_f64_s,
    /* 3 */ i32_trunc_sat_f64_u,
    /* 4 */ i64_trunc_sat_f32_s,
    /* 5 */ i64_trunc_sat_f32_u,
    /* 6 */ i64_trunc_sat_f64_s,
    /* 7 */ i64_trunc_sat_f64_u,

    /* 8 */ memory_init,
    /* 9 */ data_drop,
    /* 10 */ memory_copy,
    /* 11 */ memory_fill,

    /* 12 */ table_init,
    /* 13 */ elem_drop,
    /* 14 */ table_copy,
    /* 15 */ table_grow,
    /* 16 */ table_size,
    /* 17 */ table_fill,
];

#[rustfmt::skip]
#[allow(non_upper_case_globals)]
pub(crate) const LVL2_JMP_TABLE_0xFD: [fn(&mut Context, &mut WasmStreamReader, &mut InstructionEncoder) -> ParseResult; 0] = [/* TODO */];

fn e(_: &mut Context, _: &mut WasmStreamReader, _: &mut InstructionEncoder) -> ParseResult {
    Err(ParserError::InvalidOpcode)
}

fn lvl2_instruction_gp(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let opcode = i.read_leb128::<u32>()? as usize;
    if opcode >= LVL2_JMP_TABLE_0xFC.len() {
        return Err(ParserError::InvalidEncoding);
    }
    LVL2_JMP_TABLE_0xFC[opcode](ctxt, i, o)
}

fn lvl2_instruction_vec(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let opcode = i.read_leb128::<u32>()? as usize;
    if opcode >= LVL2_JMP_TABLE_0xFD.len() {
        return Err(ParserError::InvalidEncoding);
    }
    LVL2_JMP_TABLE_0xFD[opcode](ctxt, i, o)
}
