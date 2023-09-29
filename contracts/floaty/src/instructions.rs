use cosmwasm_schema::cw_serde;
use rand_chacha::rand_core::RngCore;

use crate::floats::{random_f32, random_f64};

/// Not intended for direct usage
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_macros))]
macro_rules! run_instr {
    ($instr:expr, $input:expr, $input_ty:ty, $return_ty:ty) => {{
        let input: $input_ty = $input;
        let ret: $return_ty;
        unsafe {
            core::arch::asm!("local.get {0}", $instr, "local.set {1}", in(local) input, out(local) ret)
        };
        ret
    }};
    ($instr:expr, $input1:expr, $input1_ty:ty, $input2:expr, $input2_ty:ty, $return_ty:ty) => {{
        let input1: $input1_ty = $input1;
        let input2: $input2_ty = $input2;
        let ret: $return_ty;
        unsafe {
            core::arch::asm!("local.get {0}", "local.get {1}", $instr, "local.set {2}", in(local) input1, in(local) input2, out(local) ret)
        };
        ret
    }};
}
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_imports))]
pub(crate) use run_instr;

/// Helper to run a single WebAssembly instruction in a type-safe way
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_macros))]
macro_rules! run {
    ("f32.eq", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.eq", $input1, f32, $input2, f32, u32)
    };
    ("f32.ne", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.ne", $input1, f32, $input2, f32, u32)
    };
    ("f32.lt", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.lt", $input1, f32, $input2, f32, u32)
    };
    ("f32.gt", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.gt", $input1, f32, $input2, f32, u32)
    };
    ("f32.le", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.le", $input1, f32, $input2, f32, u32)
    };
    ("f32.ge", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.ge", $input1, f32, $input2, f32, u32)
    };
    ("f64.eq", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.eq", $input1, f64, $input2, f64, u32)
    };
    ("f64.ne", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.ne", $input1, f64, $input2, f64, u32)
    };
    ("f64.lt", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.lt", $input1, f64, $input2, f64, u32)
    };
    ("f64.gt", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.gt", $input1, f64, $input2, f64, u32)
    };
    ("f64.le", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.le", $input1, f64, $input2, f64, u32)
    };
    ("f64.ge", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.ge", $input1, f64, $input2, f64, u32)
    };
    //
    ("f32.abs", $input:expr) => {
        $crate::instructions::run_instr!("f32.abs", $input, f32, f32)
    };
    ("f32.neg", $input:expr) => {
        $crate::instructions::run_instr!("f32.neg", $input, f32, f32)
    };
    ("f32.ceil", $input:expr) => {
        $crate::instructions::run_instr!("f32.ceil", $input, f32, f32)
    };
    ("f32.floor", $input:expr) => {
        $crate::instructions::run_instr!("f32.floor", $input, f32, f32)
    };
    ("f32.trunc", $input:expr) => {
        $crate::instructions::run_instr!("f32.trunc", $input, f32, f32)
    };
    ("f32.nearest", $input:expr) => {
        $crate::instructions::run_instr!("f32.nearest", $input, f32, f32)
    };
    ("f32.sqrt", $input:expr) => {
        $crate::instructions::run_instr!("f32.sqrt", $input, f32, f32)
    };
    ("f32.add", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.add", $input1, f32, $input2, f32, f32)
    };
    ("f32.sub", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.sub", $input1, f32, $input2, f32, f32)
    };
    ("f32.mul", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.mul", $input1, f32, $input2, f32, f32)
    };
    ("f32.div", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.div", $input1, f32, $input2, f32, f32)
    };
    ("f32.min", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.min", $input1, f32, $input2, f32, f32)
    };
    ("f32.max", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.max", $input1, f32, $input2, f32, f32)
    };
    ("f32.copysign", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f32.copysign", $input1, f32, $input2, f32, f32)
    };
    ("f64.abs", $input:expr) => {
        $crate::instructions::run_instr!("f64.abs", $input, f64, f64)
    };
    ("f64.neg", $input:expr) => {
        $crate::instructions::run_instr!("f64.neg", $input, f64, f64)
    };
    ("f64.ceil", $input:expr) => {
        $crate::instructions::run_instr!("f64.ceil", $input, f64, f64)
    };
    ("f64.floor", $input:expr) => {
        $crate::instructions::run_instr!("f64.floor", $input, f64, f64)
    };
    ("f64.trunc", $input:expr) => {
        $crate::instructions::run_instr!("f64.trunc", $input, f64, f64)
    };
    ("f64.nearest", $input:expr) => {
        $crate::instructions::run_instr!("f64.nearest", $input, f64, f64)
    };
    ("f64.sqrt", $input:expr) => {
        $crate::instructions::run_instr!("f64.sqrt", $input, f64, f64)
    };
    ("f64.add", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.add", $input1, f64, $input2, f64, f64)
    };
    ("f64.sub", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.sub", $input1, f64, $input2, f64, f64)
    };
    ("f64.mul", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.mul", $input1, f64, $input2, f64, f64)
    };
    ("f64.div", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.div", $input1, f64, $input2, f64, f64)
    };
    ("f64.min", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.min", $input1, f64, $input2, f64, f64)
    };
    ("f64.max", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.max", $input1, f64, $input2, f64, f64)
    };
    ("f64.copysign", $input1:expr, $input2:expr) => {
        $crate::instructions::run_instr!("f64.copysign", $input1, f64, $input2, f64, f64)
    };
    //
    ("i32.trunc_f32_s", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_f32_s", $input, f32, i32)
    };
    ("i32.trunc_f32_u", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_f32_u", $input, f32, u32)
    };
    ("i32.trunc_f64_s", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_f64_s", $input, f64, i32)
    };
    ("i32.trunc_f64_u", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_f64_u", $input, f64, u32)
    };
    //
    ("i64.trunc_f32_s", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_f32_s", $input, f32, i64)
    };
    ("i64.trunc_f32_u", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_f32_u", $input, f32, u64)
    };
    ("i64.trunc_f64_s", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_f64_s", $input, f64, i64)
    };
    ("i64.trunc_f64_u", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_f64_u", $input, f64, u64)
    };
    //
    ("f32.convert_i32_s", $input:expr) => {
        $crate::instructions::run_instr!("f32.convert_i32_s", $input, i32, f32)
    };
    ("f32.convert_i32_u", $input:expr) => {
        $crate::instructions::run_instr!("f32.convert_i32_u", $input, u32, f32)
    };
    ("f32.convert_i64_s", $input:expr) => {
        $crate::instructions::run_instr!("f32.convert_i64_s", $input, i64, f32)
    };
    ("f32.convert_i64_u", $input:expr) => {
        $crate::instructions::run_instr!("f32.convert_i64_u", $input, u64, f32)
    };
    ("f32.demote_f64", $input:expr) => {
        $crate::instructions::run_instr!("f32.demote_f64", $input, f64, f32)
    };
    ("f64.convert_i32_s", $input:expr) => {
        $crate::instructions::run_instr!("f64.convert_i32_s", $input, i32, f64)
    };
    ("f64.convert_i32_u", $input:expr) => {
        $crate::instructions::run_instr!("f64.convert_i32_u", $input, u32, f64)
    };
    ("f64.convert_i64_s", $input:expr) => {
        $crate::instructions::run_instr!("f64.convert_i64_s", $input, i64, f64)
    };
    ("f64.convert_i64_u", $input:expr) => {
        $crate::instructions::run_instr!("f64.convert_i64_u", $input, u64, f64)
    };
    ("f64.promote_f32", $input:expr) => {
        $crate::instructions::run_instr!("f64.promote_f32", $input, f32, f64)
    };
    //
    ("i32.reinterpret_f32", $input:expr) => {
        $crate::instructions::run_instr!("i32.reinterpret_f32", $input, f32, i32)
    };
    ("i64.reinterpret_f64", $input:expr) => {
        $crate::instructions::run_instr!("i64.reinterpret_f64", $input, f64, i64)
    };
    ("f32.reinterpret_i32", $input:expr) => {
        $crate::instructions::run_instr!("f32.reinterpret_i32", $input, u32, f32)
    };
    ("f64.reinterpret_i64", $input:expr) => {
        $crate::instructions::run_instr!("f64.reinterpret_i64", $input, u64, f64)
    };
    //
    ("i32.trunc_sat_f32_s", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f32_s", $input, f32, i32)
    };
    ("i32.trunc_sat_f32_u", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f32_u", $input, f32, u32)
    };
    ("i32.trunc_sat_f64_s", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f64_s", $input, f64, i32)
    };
    ("i32.trunc_sat_f64_u", $input:expr) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f64_u", $input, f64, u32)
    };
    ("i64.trunc_sat_f32_s", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f32_s", $input, f32, i64)
    };
    ("i64.trunc_sat_f32_u", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f32_u", $input, f32, u64)
    };
    ("i64.trunc_sat_f64_s", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f64_s", $input, f64, i64)
    };
    ("i64.trunc_sat_f64_u", $input:expr) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f64_u", $input, f64, u64)
    };
}
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_imports))]
pub(crate) use run;

#[cw_serde]
pub enum Value {
    U32(u32),
    U64(u64),
    F32(u32),
    F64(u64),
}

impl Value {
    pub fn u32(&self) -> u32 {
        match self {
            Self::U32(x) => *x,
            v => panic!("expected u32, got {:?}", v),
        }
    }

    pub fn u64(&self) -> u64 {
        match self {
            Self::U64(x) => *x,
            v => panic!("expected u64, got {:?}", v),
        }
    }

    pub fn f32(&self) -> f32 {
        match self {
            Self::F32(x) => f32::from_bits(*x),
            v => panic!("expected f32, got {:?}", v),
        }
    }

    pub fn f64(&self) -> f64 {
        match self {
            Self::F64(x) => f64::from_bits(*x),
            v => panic!("expected f64, got {:?}", v),
        }
    }
}

/// Runs the given instruction with random inputs
#[cfg(target_arch = "wasm32")]
pub fn run_instruction(instr: &str, args: &[Value]) -> Value {
    use Value::*;

    let arg1 = || args.get(0).unwrap();
    let arg2 = || args.get(0).unwrap();

    match instr {
        "f32.eq" => U32(run!("f32.eq", arg1().f32(), arg2().f32())),
        "f32.ne" => U32(run!("f32.ne", arg1().f32(), arg2().f32())),
        "f32.lt" => U32(run!("f32.lt", arg1().f32(), arg2().f32())),
        "f32.gt" => U32(run!("f32.gt", arg1().f32(), arg2().f32())),
        "f32.le" => U32(run!("f32.le", arg1().f32(), arg2().f32())),
        "f32.ge" => U32(run!("f32.ge", arg1().f32(), arg2().f32())),
        "f64.eq" => U32(run!("f64.eq", arg1().f64(), arg2().f64())),
        "f64.ne" => U32(run!("f64.ne", arg1().f64(), arg2().f64())),
        "f64.lt" => U32(run!("f64.lt", arg1().f64(), arg2().f64())),
        "f64.gt" => U32(run!("f64.gt", arg1().f64(), arg2().f64())),
        "f64.le" => U32(run!("f64.le", arg1().f64(), arg2().f64())),
        "f64.ge" => U32(run!("f64.ge", arg1().f64(), arg2().f64())),
        //
        "f32.abs" => U32(run!("f32.abs", arg1().f32()).to_bits()),
        "f32.neg" => U32(run!("f32.neg", arg1().f32()).to_bits()),
        "f32.ceil" => U32(run!("f32.ceil", arg1().f32()).to_bits()),
        "f32.floor" => U32(run!("f32.floor", arg1().f32()).to_bits()),
        "f32.trunc" => U32(run!("f32.trunc", arg1().f32()).to_bits()),
        "f32.nearest" => U32(run!("f32.nearest", arg1().f32()).to_bits()),
        "f32.sqrt" => U32(run!("f32.sqrt", arg1().f32()).to_bits()),
        "f32.add" => U32(run!("f32.add", arg1().f32(), arg2().f32()).to_bits()),
        "f32.sub" => U32(run!("f32.sub", arg1().f32(), arg2().f32()).to_bits()),
        "f32.mul" => U32(run!("f32.mul", arg1().f32(), arg2().f32()).to_bits()),
        "f32.div" => U32(run!("f32.div", arg1().f32(), arg2().f32()).to_bits()),
        "f32.min" => U32(run!("f32.min", arg1().f32(), arg2().f32()).to_bits()),
        "f32.max" => U32(run!("f32.max", arg1().f32(), arg2().f32()).to_bits()),
        "f32.copysign" => U32(run!("f32.copysign", arg1().f32(), arg2().f32()).to_bits()),
        "f64.abs" => U64(run!("f64.abs", arg1().f64()).to_bits()),
        "f64.neg" => U64(run!("f64.neg", arg1().f64()).to_bits()),
        "f64.ceil" => U64(run!("f64.ceil", arg1().f64()).to_bits()),
        "f64.floor" => U64(run!("f64.floor", arg1().f64()).to_bits()),
        "f64.trunc" => U64(run!("f64.trunc", arg1().f64()).to_bits()),
        "f64.nearest" => U64(run!("f64.nearest", arg1().f64()).to_bits()),
        "f64.sqrt" => U64(run!("f64.sqrt", arg1().f64()).to_bits()),
        "f64.add" => U64(run!("f64.add", arg1().f64(), arg2().f64()).to_bits()),
        "f64.sub" => U64(run!("f64.sub", arg1().f64(), arg2().f64()).to_bits()),
        "f64.mul" => U64(run!("f64.mul", arg1().f64(), arg2().f64()).to_bits()),
        "f64.div" => U64(run!("f64.div", arg1().f64(), arg2().f64()).to_bits()),
        "f64.min" => U64(run!("f64.min", arg1().f64(), arg2().f64()).to_bits()),
        "f64.max" => U64(run!("f64.max", arg1().f64(), arg2().f64()).to_bits()),
        "f64.copysign" => U64(run!("f64.copysign", arg1().f64(), arg2().f64()).to_bits()),
        //
        "i32.trunc_f32_s" => U32(run!("i32.trunc_f32_s", arg1().f32()) as u32),
        "i32.trunc_f32_u" => U32(run!("i32.trunc_f32_u", arg1().f32())),
        "i32.trunc_f64_s" => U32(run!("i32.trunc_f64_s", arg1().f64()) as u32),
        "i32.trunc_f64_u" => U32(run!("i32.trunc_f64_u", arg1().f64())),
        //
        "i64.trunc_f32_s" => U64(run!("i64.trunc_f32_s", arg1().f32()) as u64),
        "i64.trunc_f32_u" => U64(run!("i64.trunc_f32_u", arg1().f32())),
        "i64.trunc_f64_s" => U64(run!("i64.trunc_f64_s", arg1().f64()) as u64),
        "i64.trunc_f64_u" => U64(run!("i64.trunc_f64_u", arg1().f64())),
        //
        "f32.convert_i32_s" => U32(run!("f32.convert_i32_s", arg1().u32() as i32).to_bits()),
        "f32.convert_i32_u" => U32(run!("f32.convert_i32_u", arg1().u32()).to_bits()),
        "f32.convert_i64_s" => U32(run!("f32.convert_i64_s", arg1().u64() as i64).to_bits()),
        "f32.convert_i64_u" => U32(run!("f32.convert_i64_u", arg1().u64()).to_bits()),
        "f32.demote_f64" => U32(run!("f32.demote_f64", arg1().f64()).to_bits()),
        "f64.convert_i32_s" => U64(run!("f64.convert_i32_s", arg1().u32() as i32).to_bits()),
        "f64.convert_i32_u" => U64(run!("f64.convert_i32_u", arg1().u32()).to_bits()),
        "f64.convert_i64_s" => U64(run!("f64.convert_i64_s", arg1().u64() as i64).to_bits()),
        "f64.convert_i64_u" => U64(run!("f64.convert_i64_u", arg1().u64()).to_bits()),
        "f64.promote_f32" => U64(run!("f64.promote_f32", arg1().f32()).to_bits()),
        //
        "i32.reinterpret_f32" => U32(run!("i32.reinterpret_f32", arg1().f32()) as u32),
        "i64.reinterpret_f64" => U64(run!("i64.reinterpret_f64", arg1().f64()) as u64),
        "f32.reinterpret_i32" => U32(run!("f32.reinterpret_i32", arg1().u32()).to_bits() as u32),
        "f64.reinterpret_i64" => U64(run!("f64.reinterpret_i64", arg1().u64()).to_bits() as u64),
        //
        "i32.trunc_sat_f32_s" => U32(run!("i32.trunc_sat_f32_s", arg1().f32()) as u32),
        "i32.trunc_sat_f32_u" => U32(run!("i32.trunc_sat_f32_u", arg1().f32()) as u32),
        "i32.trunc_sat_f64_s" => U32(run!("i32.trunc_sat_f64_s", arg1().f64()) as u32),
        "i32.trunc_sat_f64_u" => U32(run!("i32.trunc_sat_f64_u", arg1().f64()) as u32),
        "i64.trunc_sat_f32_s" => U64(run!("i64.trunc_sat_f32_s", arg1().f32()) as u64),
        "i64.trunc_sat_f32_u" => U64(run!("i64.trunc_sat_f32_u", arg1().f32()) as u64),
        "i64.trunc_sat_f64_s" => U64(run!("i64.trunc_sat_f64_s", arg1().f64()) as u64),
        "i64.trunc_sat_f64_u" => U64(run!("i64.trunc_sat_f64_u", arg1().f64()) as u64),
        _ => panic!("unknown instruction: {}", instr),
    }
}

pub fn random_args_for(instr: &str, rng: &mut impl RngCore) -> Vec<Value> {
    let a = random_f32(rng);
    let b = random_f32(rng);
    let c = random_f64(rng);
    let d = random_f64(rng);
    let e = rng.next_u32();
    let f = rng.next_u64();

    use Value::*;

    let f32x2 = vec![F32(a.to_bits()), F32(b.to_bits())];
    let f64x2 = vec![F64(c.to_bits()), F64(d.to_bits())];
    let f32 = vec![F32(a.to_bits())];
    let f64 = vec![F64(c.to_bits())];
    let u32 = vec![U32(e)];
    let u64 = vec![U64(f)];

    match instr {
        "f32.eq" => f32x2,
        "f32.ne" => f32x2,
        "f32.lt" => f32x2,
        "f32.gt" => f32x2,
        "f32.le" => f32x2,
        "f32.ge" => f32x2,
        "f64.eq" => f64x2,
        "f64.ne" => f64x2,
        "f64.lt" => f64x2,
        "f64.gt" => f64x2,
        "f64.le" => f64x2,
        "f64.ge" => f64x2,
        //
        "f32.abs" => f32,
        "f32.neg" => f32,
        "f32.ceil" => f32,
        "f32.floor" => f32,
        "f32.trunc" => f32,
        "f32.nearest" => f32,
        "f32.sqrt" => f32,
        "f32.add" => f32x2,
        "f32.sub" => f32x2,
        "f32.mul" => f32x2,
        "f32.div" => f32x2,
        "f32.min" => f32x2,
        "f32.max" => f32x2,
        "f32.copysign" => f32x2,
        "f64.abs" => f64,
        "f64.neg" => f64,
        "f64.ceil" => f64,
        "f64.floor" => f64,
        "f64.trunc" => f64,
        "f64.nearest" => f64,
        "f64.sqrt" => f64,
        "f64.add" => f64x2,
        "f64.sub" => f64x2,
        "f64.mul" => f64x2,
        "f64.div" => f64x2,
        "f64.min" => f64x2,
        "f64.max" => f64x2,
        "f64.copysign" => f64x2,
        //
        "i32.trunc_f32_s" => f32,
        "i32.trunc_f32_u" => f32,
        "i32.trunc_f64_s" => f64,
        "i32.trunc_f64_u" => f64,
        //
        "i64.trunc_f32_s" => f32,
        "i64.trunc_f32_u" => f32,
        "i64.trunc_f64_s" => f64,
        "i64.trunc_f64_u" => f64,
        //
        "f32.convert_i32_s" => u32,
        "f32.convert_i32_u" => u32,
        "f32.convert_i64_s" => u64,
        "f32.convert_i64_u" => u64,
        "f32.demote_f64" => f64,
        "f64.convert_i32_s" => u32,
        "f64.convert_i32_u" => u32,
        "f64.convert_i64_s" => u64,
        "f64.convert_i64_u" => u64,
        "f64.promote_f32" => f32,
        //
        "i32.reinterpret_f32" => f32,
        "i64.reinterpret_f64" => f64,
        "f32.reinterpret_i32" => u32,
        "f64.reinterpret_i64" => u64,
        //
        "i32.trunc_sat_f32_s" => f32,
        "i32.trunc_sat_f32_u" => f32,
        "i32.trunc_sat_f64_s" => f64,
        "i32.trunc_sat_f64_u" => f64,
        "i64.trunc_sat_f32_s" => f32,
        "i64.trunc_sat_f32_u" => f32,
        "i64.trunc_sat_f64_s" => f64,
        "i64.trunc_sat_f64_u" => f64,
        _ => panic!("unknown instruction: {}", instr),
    }
}

pub const FLOAT_INSTRUCTIONS: [&str; 70] = [
    "f32.eq",
    "f32.ne",
    "f32.lt",
    "f32.gt",
    "f32.le",
    "f32.ge",
    "f64.eq",
    "f64.ne",
    "f64.lt",
    "f64.gt",
    "f64.le",
    "f64.ge",
    //
    "f32.abs",
    "f32.neg",
    "f32.ceil",
    "f32.floor",
    "f32.trunc",
    "f32.nearest",
    "f32.sqrt",
    "f32.add",
    "f32.sub",
    "f32.mul",
    "f32.div",
    "f32.min",
    "f32.max",
    "f32.copysign",
    "f64.abs",
    "f64.neg",
    "f64.ceil",
    "f64.floor",
    "f64.trunc",
    "f64.nearest",
    "f64.sqrt",
    "f64.add",
    "f64.sub",
    "f64.mul",
    "f64.div",
    "f64.min",
    "f64.max",
    "f64.copysign",
    //
    "i32.trunc_f32_s",
    "i32.trunc_f32_u",
    "i32.trunc_f64_s",
    "i32.trunc_f64_u",
    //
    "i64.trunc_f32_s",
    "i64.trunc_f32_u",
    "i64.trunc_f64_s",
    "i64.trunc_f64_u",
    //
    "f32.convert_i32_s",
    "f32.convert_i32_u",
    "f32.convert_i64_s",
    "f32.convert_i64_u",
    "f32.demote_f64",
    "f64.convert_i32_s",
    "f64.convert_i32_u",
    "f64.convert_i64_s",
    "f64.convert_i64_u",
    "f64.promote_f32",
    //
    "i32.reinterpret_f32",
    "i64.reinterpret_f64",
    "f32.reinterpret_i32",
    "f64.reinterpret_i64",
    //
    "i32.trunc_sat_f32_s",
    "i32.trunc_sat_f32_u",
    "i32.trunc_sat_f64_s",
    "i32.trunc_sat_f64_u",
    "i64.trunc_sat_f32_s",
    "i64.trunc_sat_f32_u",
    "i64.trunc_sat_f64_s",
    "i64.trunc_sat_f64_u",
];
