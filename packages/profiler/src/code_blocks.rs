use std::collections::HashMap;
use std::hash::Hash;

use loupe::MemoryUsage;

use crate::operators::OperatorSymbol;

#[derive(Debug, Hash, PartialEq, Eq, MemoryUsage, Copy, Clone)]
pub struct BlockId(pub u64);

impl BlockId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for BlockId {
    fn from(hash: u64) -> Self {
        Self(hash)
    }
}

impl PartialEq<u64> for BlockId {
    fn eq(&self, rhs: &u64) -> bool {
        self.0 == *rhs
    }
}

impl PartialEq<BlockId> for u64 {
    fn eq(&self, rhs: &BlockId) -> bool {
        rhs.0 == *self
    }
}

/// Stores non-branching Wasm code blocks so that the exact
/// list of operators can be looked up by hash later.
#[derive(Debug, MemoryUsage)]
pub struct BlockStore {
    inner: HashMap<BlockId, CodeBlock>,
}

impl BlockStore {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Register a new code block in the store. Returns a hash that can be later
    /// used to get the code block.
    pub fn register_block(&mut self, block: impl Into<CodeBlock>) -> BlockId {
        let block = block.into();
        let hash = block.get_hash();

        // let hash = calculate_hash(&v);
        self.inner.insert(hash, block);
        hash
    }

    /// Get a code block by hash.
    pub fn get_block(&self, hash: impl Into<BlockId>) -> Option<&CodeBlock> {
        self.inner.get(&hash.into())
    }
}

/// Represents a non-branching Wasm code block.
#[derive(Debug, MemoryUsage, Hash, PartialEq)]
pub struct CodeBlock {
    inner: Vec<OperatorSymbol>,
}

impl CodeBlock {
    pub fn get_hash(&self) -> BlockId {
        use std::hash::Hasher as _;

        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut s);
        BlockId(s.finish())
    }
}

impl<'b, Op> From<&'b [Op]> for CodeBlock
where
    &'b Op: Into<OperatorSymbol>,
{
    fn from(ops: &'b [Op]) -> Self {
        Self {
            inner: ops.iter().map(|item| item.into()).collect(),
        }
    }
}

impl From<Vec<OperatorSymbol>> for CodeBlock {
    fn from(ops: Vec<OperatorSymbol>) -> Self {
        Self { inner: ops }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasmer::wasmparser::{Operator, Type, TypeOrFuncType};

    #[test]
    fn block_store() {
        let mut store = BlockStore::new();

        let code_block1 = [
            Operator::GlobalGet { global_index: 333 },
            Operator::I64Const { value: 555 as i64 },
            Operator::I64LtU,
            Operator::If {
                ty: TypeOrFuncType::Type(Type::EmptyBlockType),
            },
            Operator::I32Const { value: 1 },
            Operator::GlobalSet { global_index: 222 },
            Operator::Unreachable,
            Operator::End,
        ];
        let code_block2 = [
            Operator::GlobalGet { global_index: 333 },
            Operator::I64Const { value: 222 },
            Operator::I64Sub,
            Operator::GlobalSet { global_index: 333 },
        ];

        let code_block1_hash = store.register_block(&code_block1[..]);
        let code_block2_hash = store.register_block(&code_block2[..]);
        let code_block1_another_hash = store.register_block(&code_block1[..]);

        assert_eq!(code_block1_hash, code_block1_another_hash);
        assert_ne!(code_block1_hash, code_block2_hash);

        let cb1_expected = CodeBlock::from(vec![
            OperatorSymbol::GlobalGet,
            OperatorSymbol::I64Const,
            OperatorSymbol::I64LtU,
            OperatorSymbol::If,
            OperatorSymbol::I32Const,
            OperatorSymbol::GlobalSet,
            OperatorSymbol::Unreachable,
            OperatorSymbol::End,
        ]);

        let cb2_expected = CodeBlock::from(vec![
            OperatorSymbol::GlobalGet,
            OperatorSymbol::I64Const,
            OperatorSymbol::I64Sub,
            OperatorSymbol::GlobalSet,
        ]);

        assert_eq!(store.get_block(code_block1_hash), Some(&cb1_expected));
        assert_eq!(store.get_block(code_block2_hash), Some(&cb2_expected));
        assert_eq!(store.get_block(234), None);
    }
}
