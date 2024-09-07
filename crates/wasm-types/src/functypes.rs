use crate::{NumType, RefType, ValType};
use bitcode::{Decode, Encode};
use once_cell::sync::Lazy;
use std::fmt::{self, Display, Formatter};
use std::sync::RwLock;
use std::{collections::HashMap, sync::atomic::AtomicU64};

// https://webassembly.github.io/spec/core/syntax/types.html#result-types
#[derive(Debug, Clone, Copy, Decode, Encode)]
pub struct FuncType(u64);

#[derive(Default)]
pub struct FuncTypeBuilder {
    tmp_func_type: u64,
    cnt_in: usize,
    cnt_out: usize,
    overflow: Option<(Vec<ValType>, Vec<ValType>)>,
}

pub struct FuncTypeIter {
    idx: u64,
    cnt: usize,
    kind: FuncTypeIterKind,
}

// ---------------------------- impl FuncType ----------------------------

impl PartialEq for FuncType {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FuncType {}

impl Display for FuncType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] -> [{}]",
            self.params_iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.results_iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl FuncType {
    pub fn params_iter(&self) -> FuncTypeIter {
        FuncTypeIter {
            // extract param-idx part and keep overflow marker
            idx: if self.0 & OVERFLOW_MARKER != 0 {
                self.0
            } else {
                (self.0 >> PARAM_OFFSET_LOG) & PARAM_INDEX_BITS
            },
            cnt: 0,
            kind: FuncTypeIterKind::Param,
        }
    }

    pub fn results_iter(&self) -> FuncTypeIter {
        FuncTypeIter {
            // extract result-idx part and keep overflow marker
            idx: if self.0 & OVERFLOW_MARKER != 0 {
                self.0
            } else {
                (self.0 >> RESULT_OFFSET_LOG) & RESULT_INDEX_BITS
            },
            cnt: 0,
            kind: FuncTypeIterKind::Result,
        }
    }

    fn count(mut idx: u64, max_num: usize) -> usize {
        let mut cnt = 0;
        while idx != 0 {
            // remove the one "void"-slot
            idx -= 1;
            // test next valtype by "dividing it off"
            let valtype_idx = idx / GEOMETRIC_PARTIAL_SUM_LOOKUP[max_num - cnt];
            // remove the just tested valtype from idx
            idx -= valtype_idx * GEOMETRIC_PARTIAL_SUM_LOOKUP[max_num - cnt];
            cnt += 1;
        }
        cnt
    }

    pub fn num_params(&self) -> usize {
        if self.0 & OVERFLOW_MARKER != 0 {
            // shortcut to overflow storage
            FUNCTYPES.read().unwrap().lookup(self.0).0.len()
        } else {
            Self::count(
                self.0 >> PARAM_OFFSET_LOG & PARAM_INDEX_BITS,
                MAX_NUM_PARAMS,
            )
        }
    }

    pub fn num_results(&self) -> usize {
        if self.0 & OVERFLOW_MARKER != 0 {
            // shortcut to overflow storage
            FUNCTYPES.read().unwrap().lookup(self.0).1.len()
        } else {
            Self::count(
                self.0 >> RESULT_OFFSET_LOG & RESULT_INDEX_BITS,
                MAX_NUM_RESULTS,
            )
        }
    }

    pub fn type_iter(&self) -> (FuncTypeIter, FuncTypeIter) {
        (self.params_iter(), self.results_iter())
    }

    pub fn params(&self) -> Vec<ValType> {
        if self.0 & OVERFLOW_MARKER != 0 {
            // shortcut to overflow storage
            FUNCTYPES.read().unwrap().lookup(self.0).0.clone()
        } else {
            self.params_iter().collect()
        }
    }

    pub fn results(&self) -> Vec<ValType> {
        if self.0 & OVERFLOW_MARKER != 0 {
            // shortcut to overflow storage
            FUNCTYPES.read().unwrap().lookup(self.0).1.clone()
        } else {
            self.results_iter().collect()
        }
    }

    pub fn r#type(&self) -> (Vec<ValType>, Vec<ValType>) {
        (self.params(), self.results())
    }
}

// ---------------------------- impl FuncTypeIter ----------------------------
enum FuncTypeIterKind {
    Param,
    Result,
}

impl Iterator for FuncTypeIter {
    type Item = ValType;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.idx & OVERFLOW_MARKER != 0 {
            let overflow_reader = FUNCTYPES.read().unwrap();
            let overflow_record = overflow_reader.lookup(self.idx);
            match self.kind {
                FuncTypeIterKind::Param => overflow_record.0.get(self.cnt).cloned(),
                FuncTypeIterKind::Result => overflow_record.1.get(self.cnt).cloned(),
            }
        } else {
            if self.idx == 0 {
                // -> no more params
                return None;
            }
            // remove the one "void"-slot
            self.idx -= 1;
            // test next valtype by "dividing it off"
            let valtype_idx = self.idx
                / GEOMETRIC_PARTIAL_SUM_LOOKUP[match self.kind {
                    FuncTypeIterKind::Param => MAX_NUM_PARAMS,
                    FuncTypeIterKind::Result => MAX_NUM_RESULTS,
                } - self.cnt];
            let res = valtype_of_idx(valtype_idx);
            // remove the just tested valtype from idx
            self.idx -= valtype_idx
                * GEOMETRIC_PARTIAL_SUM_LOOKUP[match self.kind {
                    FuncTypeIterKind::Param => MAX_NUM_PARAMS,
                    FuncTypeIterKind::Result => MAX_NUM_RESULTS,
                } - self.cnt];
            Some(res)
        };
        self.cnt += 1;
        res
    }
}

// ---------------------------- impl FuncTypeStorage ----------------------------

pub(crate) static FUNCTYPES: Lazy<RwLock<FuncTypeStorage>> =
    Lazy::new(|| RwLock::new(FuncTypeStorage::default()));

pub(crate) struct FuncTypeStorage {
    overflow1: HashMap<(Vec<ValType>, Vec<ValType>), u64>,
    overflow2: HashMap<u64, (Vec<ValType>, Vec<ValType>)>,
    next_id: AtomicU64,
}

impl FuncTypeStorage {
    pub(crate) fn store(&mut self, func_type: (Vec<ValType>, Vec<ValType>)) -> u64 {
        if let Some(id) = self.overflow1.get(&func_type) {
            return *id;
        }
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if id < u64::MAX {
            self.overflow1.insert(func_type.clone(), id);
            self.overflow2.insert(id, func_type);
            return id;
        }
        panic!("Overflow type storage overflow");
    }

    pub(crate) fn lookup(&self, id: u64) -> &(Vec<ValType>, Vec<ValType>) {
        if let Some(func_type) = self.overflow2.get(&id) {
            return func_type;
        }
        panic!("Type not found in storage");
    }
}

impl Default for FuncTypeStorage {
    fn default() -> Self {
        Self {
            overflow1: HashMap::new(),
            overflow2: HashMap::new(),
            next_id: AtomicU64::new(OVERFLOW_MARKER),
        }
    }
}

// ---------------------------- impl FuncTypeBuilder ----------------------------

const MAX_NUM_PARAMS: usize = 20;
const MAX_NUM_RESULTS: usize = 2;

const PARAM_OFFSET_LOG: u64 = 0;
// 57 = ceil(log2(max_param_index))
const RESULT_OFFSET_LOG: u64 = 57;

const PARAM_INDEX_BITS: u64 = 2_u64.pow((RESULT_OFFSET_LOG - PARAM_OFFSET_LOG) as u32) - 1;
const RESULT_INDEX_BITS: u64 = 2_u64.pow((63 - RESULT_OFFSET_LOG) as u32) - 1;

const OVERFLOW_MARKER: u64 = 2_u64.pow(63);

impl FuncTypeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Shorthand for `FuncTypeBuilder::new().with_params(params).with_results(results).finish()`
    pub fn create(params: &[ValType], results: &[ValType]) -> FuncType {
        FuncTypeBuilder::new()
            .with_params(params)
            .with_results(results)
            .finish()
    }

    pub fn with_params(mut self, params: &[ValType]) -> Self {
        for param in params {
            self = self.add_param(*param);
        }
        self
    }

    pub fn add_param(mut self, val_type: ValType) -> Self {
        if self.overflow.is_none() && self.cnt_in >= MAX_NUM_PARAMS {
            let partial_type = FuncType(self.tmp_func_type).r#type();
            self.overflow = Some(partial_type);
        }

        if self.overflow.is_some() {
            self.overflow.as_mut().unwrap().0.push(val_type);
        } else {
            let old_idx = (self.tmp_func_type >> PARAM_OFFSET_LOG) & PARAM_INDEX_BITS;
            let new_idx = old_idx
                + idx_of_valtype(val_type)
                    * GEOMETRIC_PARTIAL_SUM_LOOKUP[MAX_NUM_PARAMS - self.cnt_in]
                + 1;
            // set param idx bits zero
            self.tmp_func_type ^= old_idx << PARAM_OFFSET_LOG;
            // set new param idx
            self.tmp_func_type |= new_idx << PARAM_OFFSET_LOG;
            self.cnt_in += 1;
        }
        self
    }

    pub fn with_results(mut self, results: &[ValType]) -> Self {
        for result in results {
            self = self.add_result(*result);
        }
        self
    }

    pub fn add_result(mut self, val_type: ValType) -> Self {
        if self.overflow.is_none() && self.cnt_out >= MAX_NUM_RESULTS {
            let partial_type = FuncType(self.tmp_func_type).r#type();
            self.overflow = Some(partial_type);
        }

        if self.overflow.is_some() {
            self.overflow.as_mut().unwrap().1.push(val_type);
        } else {
            let old_idx = (self.tmp_func_type >> RESULT_OFFSET_LOG) & RESULT_INDEX_BITS;
            let new_idx = old_idx
                + idx_of_valtype(val_type)
                    * GEOMETRIC_PARTIAL_SUM_LOOKUP[MAX_NUM_RESULTS - self.cnt_out]
                + 1;
            // set param idx bits zero
            self.tmp_func_type ^= old_idx << RESULT_OFFSET_LOG;
            // set new param idx
            self.tmp_func_type |= new_idx << RESULT_OFFSET_LOG;
            self.cnt_out += 1;
        }
        self
    }

    pub fn finish(self) -> FuncType {
        if let Some(overflow) = self.overflow {
            FuncType(FUNCTYPES.write().unwrap().store(overflow))
        } else {
            FuncType(self.tmp_func_type)
        }
    }
}

const fn idx_of_valtype(ty: ValType) -> u64 {
    match ty {
        ValType::Number(NumType::I32) => 0,
        ValType::Number(NumType::I64) => 1,
        ValType::Number(NumType::F32) => 2,
        ValType::Number(NumType::F64) => 3,
        ValType::Reference(RefType::FunctionReference) => 4,
        ValType::Reference(RefType::ExternReference) => 5,
        ValType::VecType => 6,
    }
}

const fn valtype_of_idx(idx: u64) -> ValType {
    match idx {
        0 => ValType::i32(),
        1 => ValType::i64(),
        2 => ValType::f32(),
        3 => ValType::f64(),
        4 => ValType::funcref(),
        5 => ValType::externref(),
        6 => ValType::vec(),
        _ => panic!("invalid idx"),
    }
}

// GEOMETRIC_PARTIAL_SUM_LOOKUP[i] = s_i for r = 7, range i in 1, 11
// https://en.wikipedia.org/wiki/Geometric_series#Sum
const GEOMETRIC_PARTIAL_SUM_LOOKUP: [u64; 21] = [
    u64::MAX,          // invalid placeholder entry
    1,                 // s_1
    8,                 // s_2
    57,                // s_3
    400,               // s_4
    2801,              // s_5
    19608,             // s_6
    137257,            // s_7
    960800,            // s_8
    6725601,           // s_9
    47079208,          // s_10
    329554457,         // s_11
    2306881200,        // s_12
    16148168401,       // s_13
    113037178808,      // s_14
    791260251657,      // s_15
    5538821761600,     // s_16
    38771752331201,    // s_17
    271402266318408,   // s_18
    1899815864228857,  // s_19
    13298711049602000, // s_20
];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{
        rngs::{self},
        Rng, SeedableRng,
    };

    fn generate_functype(rng: &mut rngs::StdRng) -> (Vec<ValType>, Vec<ValType>) {
        let mut param_types = Vec::new();
        let mut result_types = Vec::new();
        for _ in 0..rng.gen_range(0..=MAX_NUM_PARAMS + 5) {
            param_types.push(valtype_of_idx(rng.gen_range(0..7)));
        }
        for _ in 0..rng.gen_range(0..=MAX_NUM_RESULTS + 2) {
            result_types.push(valtype_of_idx(rng.gen_range(0..7)));
        }
        (param_types, result_types)
    }

    #[test]
    fn test_func_type_builder() {
        // default test
        assert_eq!(
            FuncTypeBuilder::new()
                .add_param(ValType::i32())
                .add_param(ValType::i64())
                .add_result(ValType::f32())
                .add_result(ValType::f64())
                .finish()
                .r#type(),
            (
                vec![ValType::i32(), ValType::i64()],
                vec![ValType::f32(), ValType::f64()]
            )
        );
        assert_eq!(FuncTypeBuilder::new().finish().params(), vec![]);

        // full functype test
        let mut builder = FuncTypeBuilder::new();
        for _ in 0..MAX_NUM_PARAMS {
            builder = builder.add_param(ValType::i32());
        }
        let t = builder.finish();
        assert_eq!(t.num_params(), MAX_NUM_PARAMS);
        assert_eq!(t.num_results(), 0);
        assert_eq!(t.r#type(), (vec![ValType::i32(); MAX_NUM_PARAMS], vec![]));

        // overflow test
        let mut builder = FuncTypeBuilder::new();
        for _ in 0..MAX_NUM_PARAMS + 5 {
            builder = builder.add_param(ValType::vec());
            builder = builder.add_result(ValType::vec());
        }
        let t = builder.finish();
        assert_eq!(t.num_params(), MAX_NUM_PARAMS + 5);
        assert_eq!(t.num_results(), MAX_NUM_PARAMS + 5);
        assert_eq!(
            t.r#type(),
            (
                vec![ValType::vec(); MAX_NUM_PARAMS + 5],
                vec![ValType::vec(); MAX_NUM_PARAMS + 5]
            )
        );

        // test mixed overflow
        let mut builder = FuncTypeBuilder::new();
        let params = [
            ValType::funcref(),
            ValType::funcref(),
            ValType::i64(),
            ValType::vec(),
            ValType::f64(),
            ValType::i32(),
            ValType::i32(),
            ValType::f64(),
            ValType::i32(),
            ValType::externref(),
            ValType::i64(),
        ];
        for param in params.iter() {
            builder = builder.add_param(*param);
        }
        let t = builder.finish();
        assert_eq!(t.num_params(), 11);
        assert_eq!(t.num_results(), 0);
        assert_eq!(t.params(), params);
        assert_eq!(t.params_iter().collect::<Vec<_>>(), params);
    }

    #[test]
    fn test_func_type_random() {
        let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
        for _ in 0..10000 {
            let (param_types, result_types) = generate_functype(&mut rng);
            let mut builder = FuncTypeBuilder::new();
            let mut builder2 = FuncTypeBuilder::new();
            for param_type in param_types.iter() {
                builder = builder.add_param(*param_type);
                builder2 = builder2.add_param(*param_type);
            }
            for result_type in result_types.iter() {
                builder = builder.add_result(*result_type);
                builder2 = builder2.add_result(*result_type);
            }
            let func_type = builder.finish();
            let func_type2 = builder2.finish();

            assert_eq!(func_type.params(), param_types);
            assert_eq!(func_type.params_iter().collect::<Vec<_>>(), param_types);
            assert_eq!(func_type.results(), result_types);
            assert_eq!(func_type.results_iter().collect::<Vec<_>>(), result_types);
            assert_eq!(func_type, func_type2);
        }
    }
}
