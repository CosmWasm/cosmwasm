use rand_chacha::rand_core::RngCore;

use crate::floats::{random_f32, random_f64};

/// Not intended for direct usage
macro_rules! run_instr {
    ($instr:expr, $input:ident : $input_ty:ty, $return_ty:ty) => {{
        let input: $input_ty = $input;
        let ret: $return_ty;
        unsafe {
            core::arch::asm!("local.get {0}", $instr, "local.set {1}", in(local) input, out(local) ret)
        };
        ret
    }};
    ($instr:expr, $input1:ident : $input1_ty:ty, $input2:ident : $input2_ty:ty, $returnty:ty) => {{
        let input1: $input1_ty = $input1;
        let input2: $input2_ty = $input2;
        let ret: $returnty;
        unsafe {
            core::arch::asm!("local.get {0}", "local.get {1}", $instr, "local.set {2}", in(local) input1, in(local) input2, out(local) ret)
        };
        ret
    }};
}
pub(crate) use run_instr;

/// Helper to run a single WebAssembly instruction in a type-safe way
macro_rules! run {
    ("f32.eq", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.eq", $input1 : f32, $input2 : f32, u32)
    };
    ("f32.ne", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.ne", $input1 : f32, $input2 : f32, u32)
    };
    ("f32.lt", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.lt", $input1 : f32, $input2 : f32, u32)
    };
    ("f32.gt", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.gt", $input1 : f32, $input2 : f32, u32)
    };
    ("f32.le", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.le", $input1 : f32, $input2 : f32, u32)
    };
    ("f32.ge", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.ge", $input1 : f32, $input2 : f32, u32)
    };
    ("f64.eq", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.eq", $input1 : f64, $input2 : f64, u32)
    };
    ("f64.ne", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.ne", $input1 : f64, $input2 : f64, u32)
    };
    ("f64.lt", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.lt", $input1 : f64, $input2 : f64, u32)
    };
    ("f64.gt", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.gt", $input1 : f64, $input2 : f64, u32)
    };
    ("f64.le", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.le", $input1 : f64, $input2 : f64, u32)
    };
    ("f64.ge", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.ge", $input1 : f64, $input2 : f64, u32)
    };
    //
    ("f32.abs", $input:ident) => {
        $crate::instructions::run_instr!("f32.abs", $input : f32, f32)
    };
    ("f32.neg", $input:ident) => {
        $crate::instructions::run_instr!("f32.neg", $input : f32, f32)
    };
    ("f32.ceil", $input:ident) => {
        $crate::instructions::run_instr!("f32.ceil", $input : f32, f32)
    };
    ("f32.floor", $input:ident) => {
        $crate::instructions::run_instr!("f32.floor", $input : f32, f32)
    };
    ("f32.trunc", $input:ident) => {
        $crate::instructions::run_instr!("f32.trunc", $input : f32, f32)
    };
    ("f32.nearest", $input:ident) => {
        $crate::instructions::run_instr!("f32.nearest", $input : f32, f32)
    };
    ("f32.sqrt", $input:ident) => {
        $crate::instructions::run_instr!("f32.sqrt", $input : f32, f32)
    };
    ("f32.add", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.add", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.sub", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.sub", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.mul", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.mul", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.div", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.div", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.min", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.min", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.max", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.max", $input1 : f32, $input2 : f32, f32)
    };
    ("f32.copysign", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f32.copysign", $input1 : f32, $input2 : f32, f32)
    };
    ("f64.abs", $input:ident) => {
        $crate::instructions::run_instr!("f64.abs", $input : f64, f64)
    };
    ("f64.neg", $input:ident) => {
        $crate::instructions::run_instr!("f64.neg", $input : f64, f64)
    };
    ("f64.ceil", $input:ident) => {
        $crate::instructions::run_instr!("f64.ceil", $input : f64, f64)
    };
    ("f64.floor", $input:ident) => {
        $crate::instructions::run_instr!("f64.floor", $input : f64, f64)
    };
    ("f64.trunc", $input:ident) => {
        $crate::instructions::run_instr!("f64.trunc", $input : f64, f64)
    };
    ("f64.nearest", $input:ident) => {
        $crate::instructions::run_instr!("f64.nearest", $input : f64, f64)
    };
    ("f64.sqrt", $input:ident) => {
        $crate::instructions::run_instr!("f64.sqrt", $input : f64, f64)
    };
    ("f64.add", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.add", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.sub", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.sub", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.mul", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.mul", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.div", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.div", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.min", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.min", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.max", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.max", $input1 : f64, $input2 : f64, f64)
    };
    ("f64.copysign", $input1:ident, $input2:ident) => {
        $crate::instructions::run_instr!("f64.copysign", $input1 : f64, $input2 : f64, f64)
    };
    //
    ("i32.trunc_f32_s", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_f32_s", $input : f32, i32)
    };
    ("i32.trunc_f32_u", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_f32_u", $input : f32, u32)
    };
    ("i32.trunc_f64_s", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_f64_s", $input : f64, i32)
    };
    ("i32.trunc_f64_u", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_f64_u", $input : f64, u32)
    };
    //
    ("i64.trunc_f32_s", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_f32_s", $input : f32, i64)
    };
    ("i64.trunc_f32_u", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_f32_u", $input : f32, u64)
    };
    ("i64.trunc_f64_s", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_f64_s", $input : f64, i64)
    };
    ("i64.trunc_f64_u", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_f64_u", $input : f64, u64)
    };
    //
    ("f32.convert_i32_s", $input:ident) => {
        $crate::instructions::run_instr!("f32.convert_i32_s", $input : i32, f32)
    };
    ("f32.convert_i32_u", $input:ident) => {
        $crate::instructions::run_instr!("f32.convert_i32_u", $input : u32, f32)
    };
    ("f32.convert_i64_s", $input:ident) => {
        $crate::instructions::run_instr!("f32.convert_i64_s", $input : i64, f32)
    };
    ("f32.convert_i64_u", $input:ident) => {
        $crate::instructions::run_instr!("f32.convert_i64_u", $input : u64, f32)
    };
    ("f32.demote_f64", $input:ident) => {
        $crate::instructions::run_instr!("f32.demote_f64", $input : f64, f32)
    };
    ("f64.convert_i32_s", $input:ident) => {
        $crate::instructions::run_instr!("f64.convert_i32_s", $input : i32, f64)
    };
    ("f64.convert_i32_u", $input:ident) => {
        $crate::instructions::run_instr!("f64.convert_i32_u", $input : u32, f64)
    };
    ("f64.convert_i64_s", $input:ident) => {
        $crate::instructions::run_instr!("f64.convert_i64_s", $input : i64, f64)
    };
    ("f64.convert_i64_u", $input:ident) => {
        $crate::instructions::run_instr!("f64.convert_i64_u", $input : u64, f64)
    };
    ("f64.promote_f32", $input:ident) => {
        $crate::instructions::run_instr!("f64.promote_f32", $input : f32, f64)
    };
    //
    ("i32.reinterpret_f32", $input:ident) => {
        $crate::instructions::run_instr!("i32.reinterpret_f32", $input : f32, i32)
    };
    ("i64.reinterpret_f64", $input:ident) => {
        $crate::instructions::run_instr!("i64.reinterpret_f64", $input : f64, i64)
    };
    ("f32.reinterpret_i32", $input:ident) => {
        $crate::instructions::run_instr!("f32.reinterpret_i32", $input : u32, f32)
    };
    ("f64.reinterpret_i64", $input:ident) => {
        $crate::instructions::run_instr!("f64.reinterpret_i64", $input : u64, f64)
    };
    //
    ("i32.trunc_sat_f32_s", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f32_s", $input : f32, i32)
    };
    ("i32.trunc_sat_f32_u", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f32_u", $input : f32, u32)
    };
    ("i32.trunc_sat_f64_s", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f64_s", $input : f64, i32)
    };
    ("i32.trunc_sat_f64_u", $input:ident) => {
        $crate::instructions::run_instr!("i32.trunc_sat_f64_u", $input : f64, u32)
    };
    ("i64.trunc_sat_f32_s", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f32_s", $input : f32, i64)
    };
    ("i64.trunc_sat_f32_u", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f32_u", $input : f32, u64)
    };
    ("i64.trunc_sat_f64_s", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f64_s", $input : f64, i64)
    };
    ("i64.trunc_sat_f64_u", $input:ident) => {
        $crate::instructions::run_instr!("i64.trunc_sat_f64_u", $input : f64, u64)
    };
}
pub(crate) use run;

/// Runs the given instruction with random inputs
pub fn run_instruction(instr: &str, rng: &mut impl RngCore) -> u64 {
    let a = random_f32(rng);
    let b = random_f32(rng);
    let c = random_f64(rng);
    let d = random_f64(rng);
    let e = rng.next_u32();
    let f = rng.next_u64();
    let g = rng.next_u32() as i32;
    let h = rng.next_u64() as i64;

    match instr {
        "f32.eq" => run!("f32.eq", a, b) as u64,
        "f32.ne" => run!("f32.ne", a, b) as u64,
        "f32.lt" => run!("f32.lt", a, b) as u64,
        "f32.gt" => run!("f32.gt", a, b) as u64,
        "f32.le" => run!("f32.le", a, b) as u64,
        "f32.ge" => run!("f32.ge", a, b) as u64,
        "f64.eq" => run!("f64.eq", c, d) as u64,
        "f64.ne" => run!("f64.ne", c, d) as u64,
        "f64.lt" => run!("f64.lt", c, d) as u64,
        "f64.gt" => run!("f64.gt", c, d) as u64,
        "f64.le" => run!("f64.le", c, d) as u64,
        "f64.ge" => run!("f64.ge", c, d) as u64,
        //
        "f32.abs" => run!("f32.abs", a).to_bits() as u64,
        "f32.neg" => run!("f32.neg", a).to_bits() as u64,
        "f32.ceil" => run!("f32.ceil", a).to_bits() as u64,
        "f32.floor" => run!("f32.floor", a).to_bits() as u64,
        "f32.trunc" => run!("f32.trunc", a).to_bits() as u64,
        "f32.nearest" => run!("f32.nearest", a).to_bits() as u64,
        "f32.sqrt" => run!("f32.sqrt", a).to_bits() as u64,
        "f32.add" => run!("f32.add", a, b).to_bits() as u64,
        "f32.sub" => run!("f32.sub", a, b).to_bits() as u64,
        "f32.mul" => run!("f32.mul", a, b).to_bits() as u64,
        "f32.div" => run!("f32.div", a, b).to_bits() as u64,
        "f32.min" => run!("f32.min", a, b).to_bits() as u64,
        "f32.max" => run!("f32.max", a, b).to_bits() as u64,
        "f32.copysign" => run!("f32.copysign", a, b).to_bits() as u64,
        "f64.abs" => run!("f64.abs", c).to_bits(),
        "f64.neg" => run!("f64.neg", c).to_bits(),
        "f64.ceil" => run!("f64.ceil", c).to_bits(),
        "f64.floor" => run!("f64.floor", c).to_bits(),
        "f64.trunc" => run!("f64.trunc", c).to_bits(),
        "f64.nearest" => run!("f64.nearest", c).to_bits(),
        "f64.sqrt" => run!("f64.sqrt", c).to_bits(),
        "f64.add" => run!("f64.add", c, d).to_bits(),
        "f64.sub" => run!("f64.sub", c, d).to_bits(),
        "f64.mul" => run!("f64.mul", c, d).to_bits(),
        "f64.div" => run!("f64.div", c, d).to_bits(),
        "f64.min" => run!("f64.min", c, d).to_bits(),
        "f64.max" => run!("f64.max", c, d).to_bits(),
        "f64.copysign" => run!("f64.copysign", c, d).to_bits(),
        //
        "i32.trunc_f32_s" => run!("i32.trunc_f32_s", a) as u64,
        "i32.trunc_f32_u" => run!("i32.trunc_f32_u", a) as u64,
        "i32.trunc_f64_s" => run!("i32.trunc_f64_s", c) as u64,
        "i32.trunc_f64_u" => run!("i32.trunc_f64_u", c) as u64,
        //
        "i64.trunc_f32_s" => run!("i64.trunc_f32_s", a) as u64,
        "i64.trunc_f32_u" => run!("i64.trunc_f32_u", a) as u64,
        "i64.trunc_f64_s" => run!("i64.trunc_f64_s", c) as u64,
        "i64.trunc_f64_u" => run!("i64.trunc_f64_u", c) as u64,
        //
        "f32.convert_i32_s" => run!("f32.convert_i32_s", g).to_bits() as u64,
        "f32.convert_i32_u" => run!("f32.convert_i32_u", e).to_bits() as u64,
        "f32.convert_i64_s" => run!("f32.convert_i64_s", h).to_bits() as u64,
        "f32.convert_i64_u" => run!("f32.convert_i64_u", f).to_bits() as u64,
        "f32.demote_f64" => run!("f32.demote_f64", c).to_bits() as u64,
        "f64.convert_i32_s" => run!("f64.convert_i32_s", g).to_bits(),
        "f64.convert_i32_u" => run!("f64.convert_i32_u", e).to_bits(),
        "f64.convert_i64_s" => run!("f64.convert_i64_s", h).to_bits(),
        "f64.convert_i64_u" => run!("f64.convert_i64_u", f).to_bits(),
        "f64.promote_f32" => run!("f64.promote_f32", a).to_bits(),
        //
        "i32.reinterpret_f32" => run!("i32.reinterpret_f32", a) as u64,
        "i64.reinterpret_f64" => run!("i64.reinterpret_f64", c) as u64,
        "f32.reinterpret_i32" => run!("f32.reinterpret_i32", e).to_bits() as u64,
        "f64.reinterpret_i64" => run!("f64.reinterpret_i64", f).to_bits(),
        //
        "i32.trunc_sat_f32_s" => run!("i32.trunc_sat_f32_s", a) as u64,
        "i32.trunc_sat_f32_u" => run!("i32.trunc_sat_f32_u", a) as u64,
        "i32.trunc_sat_f64_s" => run!("i32.trunc_sat_f64_s", c) as u64,
        "i32.trunc_sat_f64_u" => run!("i32.trunc_sat_f64_u", c) as u64,
        "i64.trunc_sat_f32_s" => run!("i64.trunc_sat_f32_s", a) as u64,
        "i64.trunc_sat_f32_u" => run!("i64.trunc_sat_f32_u", a) as u64,
        "i64.trunc_sat_f64_s" => run!("i64.trunc_sat_f64_s", c) as u64,
        "i64.trunc_sat_f64_u" => run!("i64.trunc_sat_f64_u", c) as u64,
        _ => panic!("unknown instruction: {}", instr),
    }
}
